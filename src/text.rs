//! Plain-text version of an HTML email, for the `text/plain` part of
//! `multipart/alternative`.
//!
//! This is not a full HTML parser. It covers what email templates use: block elements
//! become line breaks, list items get a `- ` prefix, links keep their address, and
//! `head`, `style` and `script` are dropped.

/// Converts an HTML email body to readable plain text.
///
/// A link is written as `text (address)`, or only the address when the text is the address
/// itself, so links such as password resets still work in a text-only client.
pub fn html_to_text(html: &str) -> String {
    let mut out = String::with_capacity(html.len() / 2);
    let mut link: Option<Link> = None;
    let mut rest = html;

    while let Some(start) = rest.find('<') {
        push_text(&mut out, &mut link, &rest[..start]);
        rest = &rest[start..];

        if let Some(after) = rest.strip_prefix("<!--") {
            rest = after.find("-->").map_or("", |end| &after[end + 3..]);
            continue;
        }
        let Some(end) = rest.find('>') else {
            // A lone `<` is text
            push_text(&mut out, &mut link, rest);
            rest = "";
            break;
        };
        let tag = Tag::parse(&rest[1..end]);
        rest = &rest[end + 1..];

        if !tag.closing && matches!(tag.name.as_str(), "head" | "style" | "script" | "title") {
            rest = skip_element(rest, &tag.name);
            continue;
        }
        match (tag.name.as_str(), tag.closing) {
            ("br", _) => out.push('\n'),
            ("a", false) => {
                link = Some(Link {
                    href: tag.href.unwrap_or_default(),
                    text: String::new(),
                });
            }
            ("a", true) => {
                if let Some(Link { href, text }) = link.take() {
                    let text = collapse(&text);
                    let usable = !href.is_empty() && !href.starts_with('#');
                    if !usable || href == text || href.strip_prefix("mailto:") == Some(&text) {
                        out.push_str(&text);
                    } else if text.is_empty() {
                        out.push_str(&href);
                    } else {
                        out.push_str(&text);
                        out.push_str(" (");
                        out.push_str(&href);
                        out.push(')');
                    }
                }
            }
            ("li", false) => out.push_str("\n- "),
            ("td" | "th", true) => out.push(' '),
            ("hr", _) => out.push_str("\n\n"),
            (name, _) if is_block(name) => out.push_str("\n\n"),
            _ => {}
        }
    }
    push_text(&mut out, &mut link, rest);
    tidy(&out)
}

struct Link {
    href: String,
    text: String,
}

struct Tag {
    name: String,
    closing: bool,
    href: Option<String>,
}

impl Tag {
    fn parse(inner: &str) -> Tag {
        let inner = inner.trim();
        let (closing, inner) = match inner.strip_prefix('/') {
            Some(rest) => (true, rest.trim_start()),
            None => (false, inner),
        };
        let name: String = inner
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric())
            .collect::<String>()
            .to_ascii_lowercase();
        let href = if name == "a" && !closing {
            attribute(inner, "href").map(|v| decode_entities(v.trim()))
        } else {
            None
        };
        Tag {
            name,
            closing,
            href,
        }
    }
}

/// The value of `name="..."`, `name='...'` or `name=value` in a start tag.
fn attribute<'a>(tag: &'a str, name: &str) -> Option<&'a str> {
    let lower = tag.to_ascii_lowercase();
    let mut from = 0;
    while let Some(found) = lower[from..].find(name) {
        let at = from + found;
        from = at + name.len();
        let before_ok = at == 0
            || lower[..at]
                .chars()
                .next_back()
                .is_some_and(|c| c.is_ascii_whitespace());
        let after = lower[from..].trim_start();
        if !before_ok || !after.starts_with('=') {
            continue;
        }
        let value_start = tag.len() - after.len() + 1;
        let value = tag[value_start..].trim_start();
        return Some(match value.chars().next() {
            Some(q @ ('"' | '\'')) => {
                let body = &value[1..];
                &body[..body.find(q).unwrap_or(body.len())]
            }
            _ => {
                let end = value
                    .find(|c: char| c.is_ascii_whitespace() || c == '>')
                    .unwrap_or(value.len());
                &value[..end]
            }
        });
    }
    None
}

/// Skips to after `</name>`; to the end when it is missing.
fn skip_element<'a>(rest: &'a str, name: &str) -> &'a str {
    let lower = rest.to_ascii_lowercase();
    let close = format!("</{name}");
    match lower.find(&close) {
        Some(at) => rest[at..].find('>').map_or("", |end| &rest[at + end + 1..]),
        None => "",
    }
}

fn is_block(name: &str) -> bool {
    matches!(
        name,
        "p" | "div"
            | "table"
            | "tr"
            | "ul"
            | "ol"
            | "h1"
            | "h2"
            | "h3"
            | "h4"
            | "h5"
            | "h6"
            | "blockquote"
            | "section"
            | "article"
            | "header"
            | "footer"
            | "main"
            | "body"
            | "pre"
    )
}

fn push_text(out: &mut String, link: &mut Option<Link>, raw: &str) {
    if raw.is_empty() {
        return;
    }
    // Line breaks in HTML source are spaces
    let text = decode_entities(&raw.replace(['\r', '\n', '\t'], " "));
    match link {
        Some(link) => link.text.push_str(&text),
        None => out.push_str(&text),
    }
}

fn collapse(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Collapses spaces within lines, trims every line, and keeps at most one blank line.
fn tidy(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut blank = 0;
    for line in text.lines() {
        let line = line
            .split([' ', '\u{a0}'])
            .filter(|w| !w.is_empty())
            .collect::<Vec<_>>()
            .join(" ");
        if line.is_empty() {
            blank += 1;
            continue;
        }
        if !out.is_empty() {
            out.push_str(if blank > 0 { "\n\n" } else { "\n" });
        }
        blank = 0;
        out.push_str(&line);
    }
    out
}

fn decode_entities(text: &str) -> String {
    if !text.contains('&') {
        return text.to_string();
    }
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(at) = rest.find('&') {
        out.push_str(&rest[..at]);
        rest = &rest[at..];
        let decoded = rest[1..]
            .find(';')
            .filter(|&end| end <= 10)
            .and_then(|end| entity(&rest[1..=end]).map(|c| (c, end + 2)));
        match decoded {
            Some((c, len)) => {
                out.push(c);
                rest = &rest[len..];
            }
            None => {
                out.push('&');
                rest = &rest[1..];
            }
        }
    }
    out.push_str(rest);
    out
}

fn entity(name: &str) -> Option<char> {
    if let Some(num) = name.strip_prefix('#') {
        let code = match num.strip_prefix(['x', 'X']) {
            Some(hex) => u32::from_str_radix(hex, 16).ok()?,
            None => num.parse().ok()?,
        };
        return char::from_u32(code);
    }
    Some(match name {
        "amp" => '&',
        "lt" => '<',
        "gt" => '>',
        "quot" => '"',
        "apos" => '\'',
        "nbsp" => '\u{a0}',
        "copy" => '©',
        "mdash" => '—',
        "ndash" => '–',
        "hellip" => '…',
        "rsquo" => '\u{2019}',
        "lsquo" => '\u{2018}',
        "rdquo" => '\u{201d}',
        "ldquo" => '\u{201c}',
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::html_to_text;

    #[test]
    fn keeps_paragraphs_and_links() {
        let html = r#"<!doctype html><html><head><title>Reset</title>
            <style>p { color: red }</style></head>
            <body><p>Hi Ayu,</p>
            <p>Open <a href="https://app.example/reset#token=abc&amp;x=1">this link</a>
            to set a new password.</p></body></html>"#;
        assert_eq!(
            html_to_text(html),
            "Hi Ayu,\n\nOpen this link (https://app.example/reset#token=abc&x=1) to set a new password."
        );
    }

    #[test]
    fn writes_a_bare_link_once() {
        let html = r#"<p><a href='https://x.example/a'>https://x.example/a</a></p>"#;
        assert_eq!(html_to_text(html), "https://x.example/a");
        assert_eq!(
            html_to_text(r#"<a href="https://x.example/b"></a>"#),
            "https://x.example/b"
        );
        assert_eq!(
            html_to_text(r#"<a href="mailto:a@b.example">a@b.example</a>"#),
            "a@b.example"
        );
    }

    #[test]
    fn lists_breaks_and_tables() {
        let html = "<ul><li>One</li><li>Two</li></ul>Line<br>next<table><tr><td>A</td><td>B</td></tr></table>";
        assert_eq!(html_to_text(html), "- One\n- Two\n\nLine\nnext\n\nA B");
    }

    #[test]
    fn decodes_entities_and_skips_comments_and_scripts() {
        let html = "<p>5 &lt; 6 &amp;&#38; &#x41; &unknown; a&nbsp;b</p><!-- hidden --><script>alert(1)</script>";
        assert_eq!(html_to_text(html), "5 < 6 && A &unknown; a b");
    }

    #[test]
    fn plain_text_passes_through() {
        assert_eq!(html_to_text("Just text, 3 > 2"), "Just text, 3 > 2");
        assert_eq!(html_to_text("a < b"), "a < b");
    }
}
