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
use warp::Filter;

pub mod bulk;
pub mod check_email;

/// Warp filter that extracts lapin Channel.
pub fn with_channel(
	channel: Arc<Channel>,
) -> impl Filter<Extract = (Arc<Channel>,), Error = std::convert::Infallible> + Clone {
	warp::any().map(move || Arc::clone(&channel))
}
