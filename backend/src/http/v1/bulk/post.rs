use crate::config::BackendConfig;
use crate::http::v1::bulk::with_worker_db;
use crate::http::v0::check_email::post::CheckEmailRequest;
use crate::http::ReacherResponseError;
use crate::worker::consume::CHECK_EMAIL_QUEUE;
use crate::worker::do_work::{CheckEmailJobId, CheckEmailTask, TaskWebhook};
use check_if_email_exists::LOG_TARGET;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, info};
use warp::http::StatusCode;
use warp::Filter;
use crate::http::v1::bulk::task::{v1_email_verification_task, TaskPayload};
use crate::http::check_header;
use crate::http::v0::check_email::post::with_config;

#[derive(Debug, Deserialize)]
struct Request {
    input: Vec<String>,
    webhook: Option<TaskWebhook>,
    interval: Option<u64>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct Response {
    job_id: i32,
}

async fn http_handler(
    config: Arc<BackendConfig>,
    pg_pool: PgPool,
    body: Request,
) -> Result<impl warp::Reply, warp::Rejection> {
    if body.input.is_empty() {
        return Err(ReacherResponseError::new(StatusCode::BAD_REQUEST, "Empty input").into());
    }

    let res = sqlx::query("INSERT INTO v1_bulk_job (total_records) VALUES ($1) RETURNING id")
        .bind(body.input.len() as i32)
        .fetch_one(&pg_pool)
        .await
        .map_err(ReacherResponseError::from)?;
    
    let job_id: i32 = sqlx::Row::get(&res, "id");
    let n = body.input.len();
    let webhook = body.webhook.clone();
    let interval = body.interval.unwrap_or(0);

    if interval > 0 {
        for (i, to_email) in body.input.into_iter().enumerate() {
            let delay = Duration::from_secs(i as u64 * interval);
            let task_payload = TaskPayload {
                job_id,
                input: CheckEmailRequest {
                    to_email,
                    ..Default::default()
                },
            };

            v1_email_verification_task
                .builder()
                .set_json(&task_payload)
                .map_err(|e| ReacherResponseError::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
                .set_delay(delay)
                .spawn(&pg_pool)
                .await
                .map_err(|e| ReacherResponseError::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        }
        info!(target: LOG_TARGET, job_id, "Scheduled {n} emails with interval {interval}s");
    } else {
        let properties = lapin::BasicProperties::default()
            .with_content_type("application/json".into())
            .with_priority(1);

        let channel = config.must_worker_config().map_err(ReacherResponseError::from)?.channel.clone();

        for to_email in body.input {
            let input = CheckEmailRequest {
                to_email,
                ..Default::default()
            }.to_check_email_input(Arc::clone(&config));

            let task = CheckEmailTask {
                input,
                job_id: CheckEmailJobId::Bulk(job_id),
                webhook: webhook.clone(),
            };

            publish_task(Arc::clone(&channel), task, properties.clone()).await?;
        }
        info!(target: LOG_TARGET, queue = CHECK_EMAIL_QUEUE, "Added {n} emails to RabbitMQ");
    }

    Ok(warp::reply::json(&Response { job_id }))
}

pub async fn publish_task(
    channel: Arc<lapin::Channel>,
    task: CheckEmailTask,
    properties: lapin::BasicProperties,
) -> Result<(), ReacherResponseError> {
    let task_json = serde_json::to_vec(&task)?;
    channel.basic_publish("", CHECK_EMAIL_QUEUE, lapin::options::BasicPublishOptions::default(), &task_json, properties).await?.await?;
    debug!(target: LOG_TARGET, email=?task.input.to_email, queue=?CHECK_EMAIL_QUEUE, "Published task");
    Ok(())
}

pub fn v1_create_bulk_job(
    config: Arc<BackendConfig>,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("v1" / "bulk")
        .and(warp::post())
        .and(check_header(Arc::clone(&config)))
        .and(with_config(Arc::clone(&config)))
        .and(with_worker_db(config))
        .and(warp::body::content_length_limit(1024 * 1024 * 50))
        .and(warp::body::json())
        .and_then(http_handler)
}
