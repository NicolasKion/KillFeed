//! The JSON conventions of the api: its date format and whole numbers
//! written as integers.

use serde_json::Value;
use time::OffsetDateTime;
use time::macros::format_description;

/// The date format of every timestamp the api writes:
/// `2026-10-12T14:00:00.000000Z`.
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

/// A float without a fraction is written as an integer (`1.0` is `1`),
/// in the props and in the stored killmail data.
pub fn whole_numbers(value: Value) -> Value {
    match value {
        Value::Number(number) if number.is_f64() => {
            let float = number.as_f64().unwrap_or_default();
            if float.fract() == 0.0 && float.abs() < 1e15 {
                Value::from(float as i64)
            } else {
                Value::Number(number)
            }
        }
        Value::Array(items) => Value::Array(items.into_iter().map(whole_numbers).collect()),
        Value::Object(fields) => Value::Object(
            fields
                .into_iter()
                .map(|(key, value)| (key, whole_numbers(value)))
                .collect(),
        ),
        other => other,
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use time::macros::datetime;

    use super::{timestamp, whole_numbers};

    #[test]
    fn writes_dates_with_microseconds() {
        assert_eq!(
            timestamp(datetime!(2026-10-12 14:00:05 UTC)),
            "2026-10-12T14:00:05.000000Z"
        );
    }

    #[test]
    fn writes_whole_floats_as_integers() {
        assert_eq!(
            whole_numbers(json!({ "a": [1.0, 2.5, -0.0, 3], "b": 300000000000.0, "c": 1e20 })),
            json!({ "a": [1, 2.5, 0, 3], "b": 300000000000_i64, "c": 1e20 })
        );
    }
}
