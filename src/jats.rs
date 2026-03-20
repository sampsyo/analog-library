use crate::core::join;
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

/// An output text format.
enum Format {
    Html,
    Plain,
}

/// Translate JATS XML to the specified text format.
fn translate(jats: &str, fmt: Format) -> Result<String, Error> {
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
                    match fmt {
                        Format::Html => write!(out_buf, "<{html_tag}>").unwrap(),
                        Format::Plain => (),
                    }
                } else if ignore_tag(e.name().as_ref()) {
                    ignore = true;
                } else {
                    return Err(Error::UnknownTag);
                }
            }
            Event::End(e) => {
                if let Some(html_tag) = trans_tag(e.name().as_ref()) {
                    match fmt {
                        Format::Html => write!(out_buf, "</{html_tag}>").unwrap(),
                        Format::Plain => (),
                    }
                } else {
                    return Err(Error::UnknownTag);
                }
            }
            Event::Text(e) => {
                out_buf.extend_from_slice(e.as_ref());
            }
            Event::GeneralRef(e) => match fmt {
                Format::Html => {
                    out_buf.push(b'&');
                    out_buf.extend_from_slice(e.as_ref());
                    out_buf.push(b';');
                }
                Format::Plain => {
                    if let Some(c) = e.resolve_char_ref().unwrap() {
                        write!(out_buf, "{c}").unwrap();
                    }
                }
            },
            Event::Eof => break,
            _ => (),
        }
    }

    Ok(String::from_utf8(out_buf).expect("output HTML must be UTF-8"))
}

/// Translate JATS XML to HTML.
pub fn to_html(jats: &str) -> Result<String, Error> {
    translate(jats, Format::Html)
}

/// Translate JATS XML to plain text.
pub fn to_text(jats: &str) -> Result<String, Error> {
    let text = translate(jats, Format::Plain)?;

    // Normalize some whitespace. This is not a terribly efficient way to do it,
    // but what are you going to do, you know? Life is short.
    let text = join(text.split("\n").map(|line| line.trim()), "\n");
    Ok(join(
        text.trim()
            .split("\n\n")
            .map(|par| join(par.split("\n").map(|line| line.trim()), " ")),
        "\n\n",
    ))
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
        assert_eq!(to_html(jats).unwrap(), "<p>hi</p>");
        assert_eq!(to_text(jats).unwrap(), "hi");
    }

    #[test]
    fn test_italic() {
        let jats = "<jats:italic>hi</jats:italic>";
        assert_eq!(to_html(jats).unwrap(), "<i>hi</i>");
        assert_eq!(to_text(jats).unwrap(), "hi");
    }

    #[test]
    fn test_bold() {
        let jats = "<jats:bold>hi</jats:bold>";
        assert_eq!(to_html(jats).unwrap(), "<b>hi</b>");
        assert_eq!(to_text(jats).unwrap(), "hi");
    }

    #[test]
    fn test_drop_title() {
        let jats = "<jats:title>foo</jats:title>bar";
        assert_eq!(to_html(jats).unwrap(), "bar");
        assert_eq!(to_text(jats).unwrap(), "bar");
    }

    #[test]
    fn test_entities() {
        let jats = "&lt;br&gt;";
        assert_eq!(to_html(jats).unwrap(), "&lt;br&gt;");
        assert_eq!(to_text(jats).unwrap(), "br");
    }
}
