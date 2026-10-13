//! The overlay options of the query string.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OverlayOptions {
    pub count: i64,
    pub hours: i64,
    pub summary: bool,
    pub meta: bool,
}

impl OverlayOptions {
    pub fn from_query(
        _query: Option<&str>,
        _max_count: i64,
        _max_hours: i64,
        _default_summary: bool,
    ) -> Self {
        todo!()
    }
}

pub fn php_intval(_text: &str) -> i64 {
    todo!()
}

pub fn php_boolean(_text: &str) -> bool {
    todo!()
}
