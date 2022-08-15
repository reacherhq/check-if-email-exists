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

pub mod bulk;
pub mod check_email;
mod version;

use super::errors;
use sqlx::{Pool, Postgres};
use warp::Filter;

pub fn create_routes(
	o: Option<Pool<Postgres>>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
	version::get::get_version()
		.or(check_email::post::post_check_email())
		// The 3 following routes will 404 if o is None.
		.or(bulk::post::create_bulk_job(o.clone()))
		.or(bulk::get::get_bulk_job_status(o.clone()))
		.or(bulk::results::get_bulk_job_result(o))
		.recover(errors::handle_rejection)
}
