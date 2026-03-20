use quick_xml::events::Event;
use quick_xml::reader::Reader;
use std::io::Write;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("XML parse error")]
    Xml(#[from] quick_xml::Error),
    #[error("unknown tag")]
    UnknownTag,
}

/// Translate JATS XML to HTML.
pub fn to_html(jats: &str) -> Result<String, Error> {
    let mut reader = Reader::from_str(jats);
    let mut out_buf = Vec::new();

    let mut ignore = false;
    loop {
        if ignore {
            if let Event::End(e) = reader.read_event()?
                && ignore_tag(e.name().as_ref())
            {
                ignore = false;
            }
            continue;
        }

        match reader.read_event()? {
            Event::Start(e) => {
                if let Some(html_tag) = trans_tag(e.name().as_ref()) {
                    write!(out_buf, "<{html_tag}>").unwrap();
                } else if ignore_tag(e.name().as_ref()) {
                    ignore = true;
                } else {
                    return Err(Error::UnknownTag);
                }
            }
            Event::End(e) => {
                if let Some(html_tag) = trans_tag(e.name().as_ref()) {
                    write!(out_buf, "</{html_tag}>").unwrap();
                } else {
                    return Err(Error::UnknownTag);
                }
            }
            Event::Text(e) => {
                out_buf.extend_from_slice(e.as_ref());
            }
            Event::Eof => break,
            _ => (),
        }
    }

    Ok(String::from_utf8(out_buf).expect("output HTML must be UTF-8"))
}

/// Translate a JATS tag to an HTML tag.
fn trans_tag(tag: &[u8]) -> Option<&'static str> {
    if tag == b"jats:p" {
        Some("p")
    } else if tag == b"jats:italic" {
        Some("i")
    } else if tag == b"jats:bold" {
        Some("b")
    } else if tag == b"jats:sub" {
        Some("sub")
    } else if tag == b"jats:sup" {
        Some("sup")
    } else if tag == b"jats:underline" {
        Some("u")
    } else if tag == b"jats:strike" {
        Some("s")
    } else if tag == b"jats:monospace" {
        Some("code")
    } else {
        None
    }
}

/// Check if we should ignore the entire contents of a given JATS tag.
fn ignore_tag(tag: &[u8]) -> bool {
    tag == b"jats:title"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_p() {
        let jats = "<jats:p>hi</jats:p>";
        let html = "<p>hi</p>";
        assert_eq!(to_html(jats).unwrap(), html);
    }

    #[test]
    fn test_italic() {
        let jats = "<jats:italic>hi</jats:italic>";
        let html = "<i>hi</i>";
        assert_eq!(to_html(jats).unwrap(), html);
    }

    #[test]
    fn test_bold() {
        let jats = "<jats:bold>hi</jats:bold>";
        let html = "<b>hi</b>";
        assert_eq!(to_html(jats).unwrap(), html);
    }

    #[test]
    fn test_drop_title() {
        let jats = "<jats:title>foo</jats:title>bar";
        let html = "bar";
        assert_eq!(to_html(jats).unwrap(), html);
    }
}
