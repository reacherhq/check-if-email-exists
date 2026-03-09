use crate::http::v0::check_email::post::CheckEmailRequest;
use crate::worker::do_work::{CheckEmailJobId, CheckEmailTask};
use sqlxmq::job;
use std::time::Duration;

#[derive(serde::Deserialize, serde::Serialize, Clone)]
pub struct TaskPayload {
    pub job_id: i32,
    pub input: CheckEmailRequest,
}

#[job(name = "v1_email_verification_task")]
pub async fn v1_email_verification_task(
    mut current_job: sqlxmq::CurrentJob,
) -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    let payload: TaskPayload = current_job.json()?.ok_or("Got empty task.")?;
    let mut config = crate::config::load_config().await?;
    config.connect().await?;
    let config = Arc::new(config);

    info!(target: LOG_TARGET, job_id=payload.job_id, email=?payload.input.to_email, "Starting bulk verification task");

    let task = CheckEmailTask {
        input: payload.input.to_check_email_input(Arc::clone(&config)),
        job_id: CheckEmailJobId::Bulk(payload.job_id),
        webhook: None,
    };

    let result = crate::worker::do_work::check_email_and_send_result(&task).await;
    let result_json = match result {
        Ok(output) => serde_json::to_value(&output)?,
        Err(e) => serde_json::to_value(&e.to_string())?,
    };
    let payload_json = serde_json::to_value(&payload.input)?;

    sqlx::query("INSERT INTO v1_task_result (job_id, payload, result) VALUES ($1, $2, $3)")
        .bind(payload.job_id)
        .bind(payload_json)
        .bind(result_json)
        .execute(current_job.pool())
        .await?;

    current_job.complete().await?;
    Ok(())
}
