use warp::Filter;

pub mod playlist;

pub fn base() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    playlist::route()
}
