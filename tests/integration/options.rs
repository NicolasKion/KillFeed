//! The query string of the overlay and demo pages: numbers are their
//! leading integer, flags the usual words for true. The full requests are
//! pinned by `pages::demo_options`; this is the table of the two
//! functions.

use killfeed::options::{OverlayOptions, leading_integer, truthy};

#[test]
fn a_number_is_its_leading_integer() {
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
        assert_eq!(leading_integer(text), expected, "{text:?}");
    }
}

#[test]
fn a_flag_is_true_for_the_usual_words() {
    for text in ["1", "true", "TRUE", "on", "On", "yes", " yes "] {
        assert!(truthy(text), "{text:?}");
    }
    for text in ["0", "false", "off", "no", "", "2", "y", "truthy"] {
        assert!(!truthy(text), "{text:?}");
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
