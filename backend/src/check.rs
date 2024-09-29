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

//! This file contains shared logic for checking one email.

use std::env;

use warp::Filter;

/// The header which holds the Reacher backend secret.
pub const REACHER_SECRET_HEADER: &str = "x-reacher-secret";

/// Warp filter to check that the header secret is correct, if the environment
/// variable `RCH_HEADER_SECRET`  is set
pub fn check_header() -> warp::filters::BoxedFilter<()> {
	let env_var = env::var("RCH_HEADER_SECRET");

	match env_var {
		Ok(secret) => {
			if secret.len() == 0 {
				return warp::any().boxed();
			}

			let secret: &'static str = Box::leak(Box::new(secret));

			warp::header::exact(REACHER_SECRET_HEADER, secret).boxed()
		}
		Err(_) => warp::any().boxed(),
	}
}
