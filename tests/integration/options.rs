//! The query string of the overlay and demo pages, read the way PHP did:
//! `$request->integer()` is `intval()` (after Laravel trimmed the value
//! and turned an empty one into null) and `$request->boolean()` is
//! `FILTER_VALIDATE_BOOLEAN`. The full requests are pinned by the golden
//! `demo-query-*` pages; this is the table of the two functions.

use killfeed::options::{OverlayOptions, php_boolean, php_intval};

#[test]
fn intval_reads_the_leading_number_like_php() {
    for (text, expected) in [
        ("10", 10),
        ("10abc", 10),
        ("1.9", 1),
        ("1e2", 100),
        ("1e1", 10),
        ("2.5e1", 25),
        ("abc", 0),
        ("", 0),
        ("-5", -5),
        ("+7", 7),
        (" 12", 12),
        ("007", 7),
        ("0x1A", 0),
        ("99999999999999999999", i64::MAX),
        ("-99999999999999999999", i64::MIN),
        (".5", 0),
        ("1e", 1),
    ] {
        assert_eq!(php_intval(text), expected, "{text:?}");
    }
}

#[test]
fn boolean_accepts_what_filter_validate_boolean_accepts() {
    for text in ["1", "true", "TRUE", "on", "On", "yes", " yes "] {
        assert!(php_boolean(text), "{text:?}");
    }
    for text in ["0", "false", "off", "no", "", "2", "y", "truthy"] {
        assert!(!php_boolean(text), "{text:?}");
    }
}

#[test]
fn options_default_and_clamp() {
    let defaults = OverlayOptions::from_query(None, 50, 168, false);
    assert_eq!(
        defaults,
        OverlayOptions {
            count: 50,
            hours: 168,
            summary: false,
            meta: true
        }
    );
    let demo = OverlayOptions::from_query(Some("count=0&hours=500&meta=no"), 50, 168, true);
    assert_eq!(
        demo,
        OverlayOptions {
            count: 1,
            hours: 168,
            summary: true,
            meta: false
        }
    );
}
