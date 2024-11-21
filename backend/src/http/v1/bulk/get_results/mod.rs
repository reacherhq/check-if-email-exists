// Reacher - Email Verification
// Copyright (C) 2018-2023 Reacher

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
use sqlx::{Executor, PgPool, Pool, Postgres, Row};
use std::convert::TryInto;
use std::iter::Iterator;
use warp::Filter;

use super::error::{v1_bulk_handle_rejection, BulkError, CsvError};
use crate::http::with_db;
use csv_helper::{CsvResponse, CsvWrapper};

mod csv_helper;

/// Defines the download format, passed in as a query param.
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum ResponseFormat {
	Json,
	Csv,
}

// limit and offset are optional in the request
// If unspecified, offset will default to 0.
#[derive(Serialize, Deserialize)]
struct Request {
	format: Option<ResponseFormat>,
	limit: Option<u64>,
	offset: Option<u64>,
}

#[derive(Serialize, Deserialize)]
struct Response {
	results: Vec<serde_json::Value>,
}

async fn http_handler(
	job_id: i32,
	pg_pool: Pool<Postgres>,
	req: Request,
) -> Result<impl warp::Reply, warp::Rejection> {
	// Throw an error if the job is still running.
	// Is there a way to combine these 2 requests in one?
	let total_records = sqlx::query!(
		r#"SELECT total_records FROM bulk_jobs WHERE id = $1;"#,
		job_id
	)
	.fetch_one(&pg_pool)
	.await
	.map_err(BulkError::from)?
	.total_records;
	let total_processed = sqlx::query!(
		r#"SELECT COUNT(*) FROM email_results WHERE job_id = $1;"#,
		job_id
	)
	.fetch_one(&pg_pool)
	.await
	.map_err(BulkError::from)?
	.count
	.unwrap_or(0);

	if total_processed < total_records as i64 {
		return Err(BulkError::JobInProgress.into());
	}

	let format = req.format.unwrap_or(ResponseFormat::Json);
	match format {
		ResponseFormat::Json => {
			let data = job_result_json(job_id, req.limit, req.offset.unwrap_or(0), pg_pool).await?;

			let reply = serde_json::to_vec(&Response { results: data }).map_err(BulkError::from)?;

			Ok(warp::reply::with_header(
				reply,
				"Content-Type",
				"application/json",
			))
		}
		ResponseFormat::Csv => {
			let data = job_result_csv(job_id, req.limit, req.offset.unwrap_or(0), pg_pool).await?;

			Ok(warp::reply::with_header(data, "Content-Type", "text/csv"))
		}
	}
}

async fn job_result_as_iter(
	job_id: i32,
	limit: Option<u64>,
	offset: u64,
	pg_pool: Pool<Postgres>,
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

	let rows = pg_pool.fetch_all(query).await.map_err(BulkError::from)?;

	Ok(Box::new(
		rows.into_iter()
			.map(|row| row.get::<serde_json::Value, &str>("result")),
	))
}

async fn job_result_json(
	job_id: i32,
	limit: Option<u64>,
	offset: u64,
	pg_pool: Pool<Postgres>,
) -> Result<Vec<serde_json::Value>, warp::Rejection> {
	// For JSON responses, we don't want ot return more than 50 results at a
	// time, to avoid having a too big payload (unless client specifies a limit)

	Ok(
		job_result_as_iter(job_id, limit.or(Some(50)), offset, pg_pool)
			.await?
			.collect(),
	)
}

async fn job_result_csv(
	job_id: i32,
	limit: Option<u64>,
	offset: u64,
	pg_pool: Pool<Postgres>,
) -> Result<Vec<u8>, warp::Rejection> {
	let rows = job_result_as_iter(job_id, limit, offset, pg_pool).await?;
	let mut wtr = WriterBuilder::new().has_headers(true).from_writer(vec![]);

	for json_value in rows {
		let result_csv: CsvResponse = CsvWrapper(json_value)
			.try_into()
			.map_err(|e: &'static str| BulkError::Csv(CsvError::Parse(e)))?;
		wtr.serialize(result_csv)
			.map_err(|e| BulkError::Csv(CsvError::CsvLib(e)))?;
	}

	let data = wtr
		.into_inner()
		.map_err(|e| BulkError::Csv(CsvError::CsvLibWriter(Box::new(e))))?;

	Ok(data)
}

pub fn v1_get_bulk_job_results(
	pg_pool: PgPool,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
	warp::path!("v0" / "bulk" / i32 / "results")
		.and(warp::get())
		.and(with_db(pg_pool))
		.and(warp::query::<Request>())
		.and_then(http_handler)
		.recover(v1_bulk_handle_rejection)
		// View access logs by setting `RUST_LOG=reacher_backend`.
		.with(warp::log(LOG_TARGET))
}
