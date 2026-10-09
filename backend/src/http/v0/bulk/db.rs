use sqlx::{Pool, Postgres};
use warp::Filter;

/// Warp filter that extracts a Pg Pool if the option is Some, or else rejects
/// with a 404.
pub fn with_db(
	o: Option<Pool<Postgres>>,
) -> impl Filter<Extract = (Pool<Postgres>,), Error = warp::Rejection> + Clone {
	warp::any().and_then(move || {
		let o = o.clone();
		async move {
			if let Some(conn_pool) = o {
				Ok(conn_pool)
			} else {
				Err(warp::reject::not_found())
			}
		}
	})
}
