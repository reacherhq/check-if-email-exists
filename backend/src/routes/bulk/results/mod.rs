// Reacher - Email Verification
// Copyright (C) 2018-2022 Reacher

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published
// by the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.

// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

//! This file implements the /bulk/{id}/results endpoints.

use check_if_email_exists::LOG_TARGET;
use csv::WriterBuilder;
use serde::{Deserialize, Serialize};
use sqlx::{Executor, Pool, Postgres, Row};
use std::convert::TryInto;
use std::iter::Iterator;
use warp::Filter;

use super::{
	db::with_db,
	error::{BulkError, CsvError},
};
use csv_helper::{CsvWrapper, JobResultCsvResponse};

mod csv_helper;

/// Defines the download format, passed in as a query param.
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum JobResultResponseFormat {
	Json,
	Csv,
}

// limit and offset are optional in the request
// If unspecified, offset will default to 0.
#[derive(Serialize, Deserialize)]
struct JobResultRequest {
	format: Option<JobResultResponseFormat>,
	limit: Option<u64>,
	offset: Option<u64>,
}

#[derive(Serialize, Deserialize)]
struct JobResultJsonResponse {
	results: Vec<serde_json::Value>,
}

async fn job_result(
	job_id: i32,
	conn_pool: Pool<Postgres>,
	req: JobResultRequest,
) -> Result<impl warp::Reply, warp::Rejection> {
	// Throw an error if the job is still running.
	// Is there a way to combine these 2 requests in one?
	let total_records = sqlx::query!(
		r#"SELECT total_records FROM bulk_jobs WHERE id = $1;"#,
		job_id
	)
	.fetch_one(&conn_pool)
	.await
	.map_err(|e| {
		log::error!(
			target: LOG_TARGET,
			"Failed to fetch total_records for [job={}] with [error={}]",
			job_id,
			e
		);
		BulkError::from(e)
	})?
	.total_records;
	let total_processed = sqlx::query!(
		r#"SELECT COUNT(*) FROM email_results WHERE job_id = $1;"#,
		job_id
	)
	.fetch_one(&conn_pool)
	.await
	.map_err(|e| {
		log::error!(
			target: LOG_TARGET,
			"Failed to get total_processed for [job={}] with [error={}]",
			job_id,
			e
		);
		BulkError::from(e)
	})?
	.count
	.unwrap_or(0);

	if total_processed < total_records as i64 {
		return Err(BulkError::JobInProgress.into());
	}

	let format = req.format.unwrap_or(JobResultResponseFormat::Json);
	match format {
		JobResultResponseFormat::Json => {
			let data =
				job_result_json(job_id, req.limit, req.offset.unwrap_or(0), conn_pool).await?;

			let reply =
				serde_json::to_vec(&JobResultJsonResponse { results: data }).map_err(|e| {
					log::error!(
						target: LOG_TARGET,
						"Failed to convert json results to string for [job={}] with [error={}]",
						job_id,
						e
					);

					BulkError::Json(e)
				})?;

			Ok(warp::reply::with_header(
				reply,
				"Content-Type",
				"application/json",
			))
		}
		JobResultResponseFormat::Csv => {
			let data =
				job_result_csv(job_id, req.limit, req.offset.unwrap_or(0), conn_pool).await?;

			Ok(warp::reply::with_header(data, "Content-Type", "text/csv"))
		}
	}
}

async fn job_result_as_iter(
	job_id: i32,
	limit: Option<u64>,
	offset: u64,
	conn_pool: Pool<Postgres>,
) -> Result<Box<dyn Iterator<Item = serde_json::Value>>, BulkError> {
	let query = sqlx::query!(
		r#"
		SELECT result FROM email_results
		WHERE job_id = $1
		ORDER BY id
		LIMIT $2 OFFSET $3
		"#,
		job_id,
		limit.map(|l| l as i64),
		offset as i64
	);

	let rows = conn_pool.fetch_all(query).await.map_err(|e| {
		log::error!(
			target: LOG_TARGET,
			"Failed to get results for [job={}] [limit={}] [offset={}] with [error={}]",
			job_id,
			limit.map(|s| s.to_string()).unwrap_or_else(|| "n/a".into()),
			offset,
			e
		);

		BulkError::from(e)
	})?;

	Ok(Box::new(
		rows.into_iter()
			.map(|row| row.get::<serde_json::Value, &str>("result")),
	))
}

async fn job_result_json(
	job_id: i32,
	limit: Option<u64>,
	offset: u64,
	conn_pool: Pool<Postgres>,
) -> Result<Vec<serde_json::Value>, warp::Rejection> {
	// For JSON responses, we don't want ot return more than 50 results at a
	// time, to avoid having a too big payload (unless client specifies a limit)

	Ok(
		job_result_as_iter(job_id, limit.or(Some(50)), offset, conn_pool)
			.await?
			.collect(),
	)
}

async fn job_result_csv(
	job_id: i32,
	limit: Option<u64>,
	offset: u64,
	conn_pool: Pool<Postgres>,
) -> Result<Vec<u8>, warp::Rejection> {
	let rows = job_result_as_iter(job_id, limit, offset, conn_pool).await?;
	let mut wtr = WriterBuilder::new().has_headers(true).from_writer(vec![]);

	for json_value in rows {
		let result_csv: JobResultCsvResponse = CsvWrapper(json_value).try_into().map_err(|e: &'static str| {
			log::error!(
				target: LOG_TARGET,
				"Failed to convert json to csv output struct for [job={}] [limit={}] [offset={}] to csv with [error={}]",
				job_id,
				limit.map(|s| s.to_string()).unwrap_or_else(|| "n/a".into()),
				offset,
				e
			);

			BulkError::Csv(CsvError::Parse(e))
		})?;
		wtr.serialize(result_csv).map_err(|e| {
			log::error!(
				target: LOG_TARGET,
				"Failed to serialize result for [job={}] [limit={}] [offset={}] to csv with [error={}]",
				job_id,
				limit.map(|s| s.to_string()).unwrap_or_else(|| "n/a".into()),
				offset,
				e
			);

			BulkError::Csv(CsvError::CsvLib(e))
		})?;
	}

	let data = wtr.into_inner().map_err(|e| {
		log::error!(
			target: LOG_TARGET,
			"Failed to convert results for [job={}] [limit={}] [offset={}] to csv with [error={}]",
			job_id,
			limit.map(|s| s.to_string()).unwrap_or_else(|| "n/a".into()),
			offset,
			e
		);

		BulkError::Csv(CsvError::CsvLibWriter(Box::new(e)))
	})?;

	Ok(data)
}

pub fn get_bulk_job_result(
	o: Option<Pool<Postgres>>,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
	warp::path!("v0" / "bulk" / i32 / "results")
		.and(warp::get())
		.and(with_db(o))
		.and(warp::query::<JobResultRequest>())
		.and_then(job_result)
		// View access logs by setting `RUST_LOG=reacher`.
		.with(warp::log(LOG_TARGET))
}
