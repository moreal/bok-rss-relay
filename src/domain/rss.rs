use std::fmt::Write;

use once_cell::sync::Lazy;
use regex::{Regex, Replacer};
use rss::Channel;

static HTML_ENTITY_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"&(amp|lt|gt|quot|#39);").unwrap());

struct Replacement;

impl Replacer for Replacement {
    fn replace_append(&mut self, caps: &regex::Captures<'_>, dst: &mut String) {
        dst.write_str(match &caps[1] {
            "amp" => "&",
            "lt" => "<",
            "gt" => ">",
            "quot" => "\"",
            "#39" => "'",
            _ => &caps[1],
        })
        .unwrap();
    }
}

fn unescape_html(s: &str) -> String {
    HTML_ENTITY_REGEX.replace_all(s, Replacement).into_owned()
}

pub(crate) fn unescape_rss(channel: Channel) -> Channel {
    let mut channel = channel.clone();
    for item in channel.items_mut() {
        item.set_title(item.title().map(unescape_html));
        item.set_content(item.content().map(unescape_html));
        item.set_description(item.description().map(unescape_html));
    }

    channel
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unescape_html_amp() {
        assert_eq!(unescape_html("&amp;"), "&");
    }

    #[test]
    fn test_unescape_html_lt() {
        assert_eq!(unescape_html("&lt;"), "<");
    }

    #[test]
    fn test_unescape_html_gt() {
        assert_eq!(unescape_html("&gt;"), ">");
    }

    #[test]
    fn test_unescape_html_quot() {
        assert_eq!(unescape_html("&quot;"), "\"");
    }

    #[test]
    fn test_unescape_html_39() {
        assert_eq!(unescape_html("&#39;"), "'");
    }

    #[test]
    fn test_unescape_html_multiple() {
        assert_eq!(unescape_html("&lt;div&gt;"), "<div>");
        assert_eq!(
            unescape_html("&lt;a href=&quot;link&quot;&gt;"),
            "<a href=\"link\">"
        );
        assert_eq!(
            unescape_html("text with &amp; and &lt; symbols"),
            "text with & and < symbols"
        );
    }

    #[test]
    fn test_unescape_html_no_entities() {
        let text = "Plain text with no HTML entities";
        assert_eq!(unescape_html(text), text);
    }

    #[test]
    fn test_unescape_html_empty_string() {
        assert_eq!(unescape_html(""), "");
    }

    #[test]
    fn test_unescape_html_complex() {
        let input: &'static str =
            "The &lt;code&gt;function&lt;/code&gt; returns &quot;value&quot; &amp; exits.";
        let expected: &'static str = "The <code>function</code> returns \"value\" & exits.";
        assert_eq!(unescape_html(input), expected);
    }
}
