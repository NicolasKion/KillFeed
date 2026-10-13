//! JSON the way the Laravel app wrote it: Eloquent's date format and
//! PHP's json_encode numbers.

use serde_json::Value;
use time::OffsetDateTime;
use time::macros::format_description;

/// Eloquent's date serialization: `2026-10-12T14:00:00.000000Z`.
pub fn timestamp(moment: OffsetDateTime) -> String {
    moment
        .to_offset(time::UtcOffset::UTC)
        .format(format_description!(
            "[year]-[month]-[day]T[hour]:[minute]:[second].[subsecond digits:6]Z"
        ))
        .expect("a timestamp formats")
}

pub fn optional_timestamp(moment: Option<OffsetDateTime>) -> Value {
    moment.map_or(Value::Null, |moment| Value::String(timestamp(moment)))
}

/// json_encode writes a float without a fraction as an integer (`1.0` is
/// `1`): the props carry the same numbers.
pub fn php_numbers(value: Value) -> Value {
    match value {
        Value::Number(number) if number.is_f64() => {
            let float = number.as_f64().unwrap_or_default();
            if float.fract() == 0.0 && float.abs() < 1e15 {
                Value::from(float as i64)
            } else {
                Value::Number(number)
            }
        }
        Value::Array(items) => Value::Array(items.into_iter().map(php_numbers).collect()),
        Value::Object(fields) => Value::Object(
            fields
                .into_iter()
                .map(|(key, value)| (key, php_numbers(value)))
                .collect(),
        ),
        other => other,
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use time::macros::datetime;

    use super::{php_numbers, timestamp};

    #[test]
    fn writes_eloquent_dates() {
        assert_eq!(
            timestamp(datetime!(2026-10-12 14:00:05 UTC)),
            "2026-10-12T14:00:05.000000Z"
        );
    }

    #[test]
    fn writes_numbers_like_json_encode() {
        assert_eq!(
            php_numbers(json!({ "a": [1.0, 2.5, -0.0, 3], "b": 300000000000.0, "c": 1e20 })),
            json!({ "a": [1, 2.5, 0, 3], "b": 300000000000_i64, "c": 1e20 })
        );
    }
}
