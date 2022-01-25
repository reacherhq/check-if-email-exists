// check-if-email-exists
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

use serde::Serializer;
use std::fmt::Display;

/// Implement the `Serialize` trait for types that are `Display`.
/// https://stackoverflow.com/questions/58103801/serialize-using-the-display-trait
pub fn ser_with_display<T, S>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
where
	T: Display,
	S: Serializer,
{
	serializer.collect_str(value)
}
