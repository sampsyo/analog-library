use quick_xml::events::{BytesEnd, BytesStart, Event};
use quick_xml::reader::Reader;
use quick_xml::writer::Writer;
use std::io::Cursor;

pub fn to_html(jats: &str) -> String {
    let mut reader = Reader::from_str(jats);
    let mut writer = Writer::new(Cursor::new(Vec::new()));
    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) if e.name().as_ref() == b"jats:p" => {
                assert!(
                    writer
                        .write_event(Event::Start(BytesStart::new("p")))
                        .is_ok()
                );
            }
            Ok(Event::End(e)) if e.name().as_ref() == b"jats:p" => {
                assert!(writer.write_event(Event::End(BytesEnd::new("p"))).is_ok());
            }
            Ok(Event::Eof) => break,
            Ok(e) => assert!(writer.write_event(e.borrow()).is_ok()),
            // TODO fail gracefully
            Err(e) => panic!("XML error at position {}: {:?}", reader.error_position(), e),
        }
    }
    String::from_utf8(writer.into_inner().into_inner()).expect("output HTML must be UTF-8")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_p() {
        let jats = "<jats:p>hi</jats:p>";
        let html = "<p>hi</p>";
        assert_eq!(to_html(jats), html);
    }
}
