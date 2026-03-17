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
