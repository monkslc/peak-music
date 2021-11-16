use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use futures::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use serde::{Deserialize, Serialize};
use tokio::sync::{broadcast, oneshot};
use warp::{ws, Filter};

type PlaylistTx = broadcast::Sender<ws::Message>;
type PlaylistRx = broadcast::Receiver<ws::Message>;
type UserTx = SplitSink<ws::WebSocket, ws::Message>;
type UserRx = SplitStream<ws::WebSocket>;
type CloseTx = oneshot::Sender<()>;
type CloseRx = oneshot::Receiver<()>;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct PlaylistResponse {
    pub name: String,
    pub user_count: usize,
}

pub fn route() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    let playlist_map = Arc::new(RwLock::new(HashMap::<String, PlaylistTx>::new()));
    let playlist_map_rest = Arc::clone(&playlist_map);

    let playlist_base = warp::path!("playlists" / String);

    let playlist_ws = playlist_base
        .map(move |playlist: String| {
            let map = playlist_map.read().unwrap();
            let pl_tx = if let Some(pl_tx) = map.get(&playlist) {
                pl_tx.clone()
            } else {
                drop(map);
                // TODO: Make sure that this won't be perpetually locked by readers coming in
                let mut map = playlist_map.write().unwrap();
                let pl_tx = map
                    .entry(playlist.clone())
                    .or_insert_with(|| broadcast::channel::<ws::Message>(20).0);
                pl_tx.clone()
            };

            (playlist, pl_tx)
        })
        .and(warp::ws())
        .map(|(playlist, pl_tx), ws: ws::Ws| {
            ws.on_upgrade(|ws| user_connected(ws, playlist, pl_tx))
        });

    let playlist_rest = playlist_base
        .and(warp::get())
        .map(move |playlist: String| {
            let map = playlist_map_rest.read().unwrap();
            let user_count = map
                .get(&playlist)
                .map(|pl_tx| pl_tx.receiver_count())
                .unwrap_or(0);

            (playlist, user_count)
        })
        .map(|(name, user_count): (String, usize)| {
            let resp = PlaylistResponse { name, user_count };
            serde_json::to_string(&resp).unwrap()
        });

    warp::any().and(playlist_ws.or(playlist_rest))
}

async fn user_connected(ws: ws::WebSocket, playlist: String, pl_tx: PlaylistTx) {
    println!("User connected to playlist: {}", playlist);

    let pl_rx = pl_tx.subscribe();
    let (user_tx, user_rx) = ws.split();
    let (close_tx, close_rx) = oneshot::channel::<()>();

    tokio::spawn(playlist_to_user(pl_rx, user_tx, close_rx));
    tokio::spawn(user_to_playlist(user_rx, pl_tx, close_tx));
}

async fn playlist_to_user(mut pl_rx: PlaylistRx, mut user_tx: UserTx, mut close_rx: CloseRx) {
    loop {
        tokio::select! {
            msg = pl_rx.recv() => {
                match msg {
                    Ok(msg) => {
                        if user_tx.send(msg).await.is_err() {
                            break
                        }
                    }
                    _ => break
                }
            }

            _ = &mut close_rx => {
                break
            }
        }
    }

    println!("Closing user connection!");
    let _ = user_tx.close().await;
}

async fn user_to_playlist(mut user_rx: UserRx, pl_tx: PlaylistTx, close_tx: CloseTx) {
    while let Some(Ok(msg)) = user_rx.next().await {
        if msg.is_text() {
            if pl_tx.send(msg).is_err() {
                break;
            }
        }
    }

    if pl_tx.receiver_count() == 1 {
        // TODO: Close the channel
        println!("This is the last user. We should drop the playlist");
    }

    let _ = close_tx.send(());
}
