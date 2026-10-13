//! HTML escaping of text and attribute values, single quotes included.

pub fn escape(text: &str) -> String {
    let mut escaped = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '"' => escaped.push_str("&quot;"),
            '\'' => escaped.push_str("&#039;"),
            c => escaped.push(c),
        }
    }
    escaped
}

#[cfg(test)]
mod tests {
    #[test]
    fn escapes_markup_and_quotes() {
        assert_eq!(
            super::escape(r#"Body & Face <"x"> 'y'"#),
            "Body &amp; Face &lt;&quot;x&quot;&gt; &#039;y&#039;"
        );
    }
}
