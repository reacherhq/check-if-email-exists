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

use sqlx::{Pool, Postgres};
use warp::Filter;

/// Warp filter that extracts a Pg Pool if the option is Some, or else rejects
/// with a 404.
pub fn with_db(
	o: Option<Pool<Postgres>>,
) -> impl Filter<Extract = (Pool<Postgres>,), Error = warp::Rejection> + Clone {
	warp::any().and_then(move || {
		let o = o.clone(); // Still not 100% sure why I need to clone here...
		async move {
			if let Some(conn_pool) = o {
				Ok(conn_pool)
			} else {
				Err(warp::reject::not_found())
			}
		}
	})
}
