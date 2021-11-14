use std::net::SocketAddr;

use crate::routes;

pub async fn start(addr: SocketAddr) {
    warp::serve(routes::base()).run(addr).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::{SinkExt, StreamExt};
    use std::net::{IpAddr, Ipv4Addr, SocketAddr};
    use std::sync::atomic::{AtomicU16, Ordering};
    use std::time::Duration;
    use tokio_tungstenite as tt;
    use tt::tungstenite as ws;

    use routes::playlist::PlaylistResponse;

    type ClientConnection = tt::WebSocketStream<tt::MaybeTlsStream<tokio::net::TcpStream>>;

    static PORT: AtomicU16 = AtomicU16::new(3030);
    pub fn next_addr() -> SocketAddr {
        let port = PORT.fetch_add(1, Ordering::Relaxed);
        SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), port)
    }

    pub fn format_ws_url(addr: SocketAddr, playlist: &str) -> String {
        format!("ws://localhost:{}/playlists/{}", addr.port(), playlist)
    }

    pub fn format_rest_url(addr: SocketAddr, playlist: &str) -> String {
        format!("http://localhost:{}/playlists/{}", addr.port(), playlist)
    }

    #[tokio::test()]
    async fn echo() {
        let playlist = "echo";
        let addr = next_addr();
        tokio::spawn(start(addr));

        let ws_url = format_ws_url(addr, playlist);
        let rest_url = format_rest_url(addr, playlist);

        let (mut conn1, _) = tt::connect_async(&ws_url).await.unwrap();
        let (mut conn2, _) = tt::connect_async(&ws_url).await.unwrap();

        assert_playlist_resp(
            &rest_url,
            PlaylistResponse {
                name: String::from(playlist),
                user_count: 2,
            },
        )
        .await;

        let test_msg = ws::Message::text("Hello, from user 1!");
        conn1.send(test_msg.clone()).await.unwrap();
        assert_recv(&mut [&mut conn1, &mut conn2], &test_msg).await;

        let test_msg = ws::Message::text("HEY! From user 2!");
        conn2.send(test_msg.clone()).await.unwrap();
        assert_recv(&mut [&mut conn1, &mut conn2], &test_msg).await;

        conn2.close(None).await.unwrap();

        assert_playlist_resp(
            &rest_url,
            PlaylistResponse {
                name: String::from(playlist),
                user_count: 1,
            },
        )
        .await;

        let test_msg = ws::Message::text("User 2 left. Now its just me :(");
        conn1.send(test_msg.clone()).await.unwrap();
        assert_recv(&mut [&mut conn1], &test_msg).await;
    }

    async fn assert_recv(clients: &mut [&mut ClientConnection], expected: &ws::Message) {
        for conn in clients {
            let msg = tokio::time::timeout(Duration::from_secs(10), conn.next())
                .await
                .unwrap()
                .unwrap()
                .unwrap();
            assert_eq!(*expected, msg);
        }
    }

    async fn assert_playlist_resp(url: &str, expected: PlaylistResponse) {
        let resp = http_get_read_all(url).await;
        let resp: PlaylistResponse = serde_json::from_str(&resp).unwrap();
        assert_eq!(expected, resp);
    }

    async fn http_get_read_all(url: &str) -> String {
        reqwest::get(url).await.unwrap().text().await.unwrap()
    }
}