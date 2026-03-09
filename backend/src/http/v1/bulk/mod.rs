use crate::config::BackendConfig;
use crate::http::ReacherResponseError;
use sqlx::PgPool;
use std::sync::Arc;
use warp::http::StatusCode;
use warp::Filter;

pub mod get_progress;
pub mod get_results;
pub mod post;
pub mod task;

pub fn with_worker_db(
    config: Arc<BackendConfig>,
) -> impl Filter<Extract = (PgPool,), Error = warp::Rejection> + Clone {
    warp::any().and_then(move || {
        let config = Arc::clone(&config);
        let pool = config.get_pg_pool();
        async move {
            if !config.worker.enable {
                return Err(warp::reject::custom(ReacherResponseError::new(
                    StatusCode::SERVICE_UNAVAILABLE,
                    "Please enable worker mode on Reacher before calling this endpoint",
                )));
            }
            pool.ok_or_else(|| {
                warp::reject::custom(ReacherResponseError::new(
                    StatusCode::SERVICE_UNAVAILABLE,
                    "Please configure a Postgres database on Reacher before calling this endpoint",
                ))
            })
        }
    })
}

pub fn v1_bulk_routes(
    config: Arc<BackendConfig>,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    post::v1_create_bulk_job(Arc::clone(&config))
        .or(get_progress::v1_get_bulk_job_progress(Arc::clone(&config)))
        .or(get_results::v1_get_bulk_job_results(config))
}
