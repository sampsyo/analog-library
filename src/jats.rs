use quick_xml::events::{BytesEnd, BytesStart, Event};
use quick_xml::reader::Reader;
use quick_xml::writer::Writer;
use std::io::Cursor;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("XML parse error")]
    Xml(#[from] quick_xml::Error),
}

/// Translate JATS XML to HTML.
pub fn to_html(jats: &str) -> Result<String, Error> {
    let mut reader = Reader::from_str(jats);
    let mut writer = Writer::new(Cursor::new(Vec::new()));

    loop {
        match reader.read_event()? {
            Event::Start(e) => {
                if let Some(html_tag) = trans_tag(e.name().as_ref()) {
                    assert!(
                        writer
                            .write_event(Event::Start(BytesStart::new(html_tag)))
                            .is_ok()
                    );
                } else {
                    panic!("unknown tag start")
                }
            }
            Event::End(e) => {
                if let Some(html_tag) = trans_tag(e.name().as_ref()) {
                    assert!(
                        writer
                            .write_event(Event::End(BytesEnd::new(html_tag)))
                            .is_ok()
                    );
                } else {
                    panic!("unknown tag end")
                }
            }
            Event::Eof => break,
            e => assert!(writer.write_event(e.borrow()).is_ok()),
        }
    }

    let bytes = writer.into_inner().into_inner();
    Ok(String::from_utf8(bytes).expect("output HTML must be UTF-8"))
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
}
