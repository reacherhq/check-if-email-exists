// check_if_email_exists
// Copyright (C) 2018-2019 Amaury Martiny

// check_if_email_exists is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// check_if_email_exists is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with check_if_email_exists.  If not, see <http://www.gnu.org/licenses/>.

use serde::Serializer;
use std::fmt::Display;

/// Implement the `Serialize` trait for types that are `Display`
/// https://stackoverflow.com/questions/58103801/serialize-using-the-display-trait
pub fn ser_with_display<T, S>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
where
	T: Display,
	S: Serializer,
{
	serializer.collect_str(value)
}
