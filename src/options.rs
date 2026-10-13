//! The overlay options of the query string (`count`, `hours`, `summary`,
//! `meta`). The last value of a repeated key wins and `key[]` makes an
//! array; every value is trimmed and an empty one counts as null. A
//! number is its leading integer (`10abc` is 10, `1e2` is 100) and a flag
//! is true for `1`, `true`, `on` or `yes`. OBS sources keep the URLs
//! streamers pasted, so every odd value has to keep meaning what it
//! means.

/// The options of an overlay or the demo, clamped to 1 and the maximums.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OverlayOptions {
    pub count: i64,
    pub hours: i64,
    pub summary: bool,
    pub meta: bool,
}

/// A query parameter, trimmed.
#[derive(Debug, Clone, PartialEq)]
enum Input {
    Missing,
    /// An empty (or blank) value.
    Null,
    Text(String),
    /// `key[]=...` or `key[x]=...`.
    Array,
}

impl OverlayOptions {
    /// Missing values take the defaults
    /// (the maximums, `summary` as given, `meta` on).
    pub fn from_query(
        query: Option<&str>,
        max_count: i64,
        max_hours: i64,
        default_summary: bool,
    ) -> Self {
        let query = query.unwrap_or_default();
        Self {
            count: integer(&input(query, "count"), max_count).clamp(1, max_count.max(1)),
            hours: integer(&input(query, "hours"), max_hours).clamp(1, max_hours.max(1)),
            summary: boolean(&input(query, "summary"), default_summary),
            meta: boolean(&input(query, "meta"), true),
        }
    }
}

/// A number option: null is 0, an array 1.
fn integer(input: &Input, default: i64) -> i64 {
    match input {
        Input::Missing => default,
        Input::Null => 0,
        Input::Text(text) => leading_integer(text),
        Input::Array => 1,
    }
}

/// A flag option: null or an array is false.
fn boolean(input: &Input, default: bool) -> bool {
    match input {
        Input::Missing => default,
        Input::Null | Input::Array => false,
        Input::Text(text) => truthy(text),
    }
}

/// The value of `key`: the last one given, trimmed, an empty one null.
fn input(query: &str, key: &str) -> Input {
    let mut found = Input::Missing;
    for pair in query.split('&').filter(|pair| !pair.is_empty()) {
        let (name, value) = pair.split_once('=').unwrap_or((pair, ""));
        let name = decode(name);
        let (base, is_array) = match name.find('[') {
            Some(bracket) if name.contains(']') => (&name[..bracket], true),
            _ => (name.as_str(), false),
        };
        if base != key {
            continue;
        }
        found = if is_array {
            Input::Array
        } else {
            let value = trim(&decode(value));
            if value.is_empty() {
                Input::Null
            } else {
                Input::Text(value)
            }
        };
    }
    found
}

/// URL decoding: `+` is a space, `%XX` a byte.
fn decode(text: &str) -> String {
    let bytes = text.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        match bytes[index] {
            b'+' => out.push(b' '),
            b'%' if index + 2 < bytes.len() => {
                let hex = |byte: u8| (byte as char).to_digit(16);
                match (hex(bytes[index + 1]), hex(bytes[index + 2])) {
                    (Some(high), Some(low)) => {
                        out.push((high * 16 + low) as u8);
                        index += 2;
                    }
                    _ => out.push(b'%'),
                }
            }
            byte => out.push(byte),
        }
        index += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// Trims whitespace and the zero width characters.
fn trim(text: &str) -> String {
    text.trim_matches(|c: char| {
        c.is_whitespace() || matches!(c, '\u{FEFF}' | '\u{200B}' | '\u{200E}')
    })
    .to_owned()
}

/// The leading number of a string (an exponent included, `1e2` is 100),
/// truncated, 0 without one, capped at the 64-bit range.
pub fn leading_integer(text: &str) -> i64 {
    let text = text.trim_start_matches([' ', '\t', '\n', '\r', '\x0b', '\x0c']);
    let bytes = text.as_bytes();
    let mut end = 0;
    if matches!(bytes.first(), Some(b'+' | b'-')) {
        end += 1;
    }
    let digits_start = end;
    while bytes.get(end).is_some_and(u8::is_ascii_digit) {
        end += 1;
    }
    let mut integral = end > digits_start;
    let mut is_float = false;
    if bytes.get(end) == Some(&b'.') {
        let mut fraction = end + 1;
        while bytes.get(fraction).is_some_and(u8::is_ascii_digit) {
            fraction += 1;
        }
        if integral || fraction > end + 1 {
            is_float = true;
            integral = true;
            end = fraction;
        }
    }
    if !integral {
        return 0;
    }
    if matches!(bytes.get(end), Some(b'e' | b'E')) {
        let mut exponent = end + 1;
        if matches!(bytes.get(exponent), Some(b'+' | b'-')) {
            exponent += 1;
        }
        let exponent_digits = exponent;
        while bytes.get(exponent).is_some_and(u8::is_ascii_digit) {
            exponent += 1;
        }
        if exponent > exponent_digits {
            is_float = true;
            end = exponent;
        }
    }
    let number = &text[..end];
    if !is_float && let Ok(value) = number.parse::<i64>() {
        return value;
    }
    // A float, or an integer beyond 64 bits: through a double, capped.
    let value: f64 = number.parse().unwrap_or(0.0);
    if value.is_nan() {
        0
    } else if value >= 9.223_372_036_854_776e18 {
        i64::MAX
    } else if value <= -9.223_372_036_854_776e18 {
        i64::MIN
    } else {
        value.trunc() as i64
    }
}

/// `1`, `true`, `on` and `yes` in any case are true, anything else
/// false.
pub fn truthy(text: &str) -> bool {
    let text = text.trim_matches([' ', '\t', '\n', '\r', '\x0b', '\0']);
    matches!(
        text.to_ascii_lowercase().as_str(),
        "1" | "true" | "on" | "yes"
    )
}

#[cfg(test)]
mod tests {
    use super::{Input, decode, input};

    #[test]
    fn reads_the_last_trimmed_value() {
        assert_eq!(
            input("count=3&count=4", "count"),
            Input::Text("4".to_owned())
        );
        assert_eq!(
            input("count=%2012%20", "count"),
            Input::Text("12".to_owned())
        );
        assert_eq!(input("hours=+7", "hours"), Input::Text("7".to_owned()));
        assert_eq!(input("count=", "count"), Input::Null);
        assert_eq!(input("count", "count"), Input::Null);
        assert_eq!(input("count[]=5", "count"), Input::Array);
        assert_eq!(input("counter=5", "count"), Input::Missing);
        assert_eq!(decode("a%2"), "a%2");
        assert_eq!(decode("%41%zz"), "A%zz");
    }
}
