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

use lapin::Channel;
use std::sync::Arc;
use warp::http::StatusCode;
use warp::Filter;

use super::ReacherResponseError;

pub mod bulk;
pub mod check_email;

/// Warp filter that extracts lapin Channel, or returns a 503 error if it's not
/// available.
pub fn with_channel(
	channel: Option<Arc<Channel>>,
) -> impl Filter<Extract = (Arc<Channel>,), Error = warp::Rejection> + Clone {
	warp::any().and_then(move || {
		let channel = channel.clone();
		async move {
			channel.ok_or_else(|| {
				warp::reject::custom(ReacherResponseError::new(
					StatusCode::SERVICE_UNAVAILABLE,
					"Please configure a RabbitMQ instance on Reacher before calling this endpoint",
				))
			})
		}
	})
}
