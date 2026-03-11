use crate::crossref;
use std::fmt::{Display, Write};

struct BibAuthors<'a>(&'a [crossref::Author]);

impl<'a> Display for BibAuthors<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut first = true;
        for a in self.0.iter() {
            if !first {
                f.write_str(" and ")?;
            }
            first = false;

            f.write_str(&a.given)?;
            f.write_char(' ')?;
            f.write_str(&a.family)?;
        }
        Ok(())
    }
}

enum BibType {
    Article,
    InProceedings,
    Misc,
}

impl BibType {
    /// Map a Crossref API type string to a BibTeX type.
    fn from_crossref(type_: &str) -> Self {
        // Maybe someday we want to handle other kinds, but these two will do
        // just fine. Falling back to `@misc` for anything else.
        if type_ == "journal-article" {
            Self::Article
        } else if type_ == "proceedings-article" {
            Self::InProceedings
        } else {
            Self::Misc
        }
    }
}

impl Display for BibType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BibType::Article => f.write_str("article"),
            BibType::InProceedings => f.write_str("inproceedings"),
            BibType::Misc => f.write_str("misc"),
        }
    }
}

pub struct Entry<'a>(pub &'a crossref::Paper);

impl<'a> Display for Entry<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Citation keys like `lamport1978`. I'm sure we can do a lot better, but
        // this is better than nothing.
        let citekey = format!(
            "{}{}",
            self.0.author[0].family.to_lowercase(),
            self.0.published.year()
        );

        let type_ = BibType::from_crossref(&self.0.type_);

        writeln!(f, "@{type_}{{{citekey},").unwrap();
        writeln!(f, "  title = {},", BibStr::verb(&self.0.title)).unwrap();
        writeln!(f, "  author = {},", BibAuthors(&self.0.author)).unwrap();
        match type_ {
            BibType::Article => {
                writeln!(f, "  journal = {},", BibStr::new(&self.0.container_title)).unwrap();
                if let Some(volume) = &self.0.volume {
                    writeln!(f, "  volume = {},", BibStr::new(volume)).unwrap();
                }
                if let Some(issue) = &self.0.issue {
                    writeln!(f, "  issue = {},", BibStr::new(issue)).unwrap();
                }
                writeln!(f, "  year = {},", self.0.published.year()).unwrap();
                if let Some(month) = self.0.published.month() {
                    writeln!(f, "  month = {},", month).unwrap();
                }
                if let Some(day) = self.0.published.day() {
                    writeln!(f, "  day = {},", day).unwrap();
                }
            }
            BibType::InProceedings => {
                if let Some(venue) = &self.0.event {
                    writeln!(f, "  booktitle = {},", BibStr::new(venue)).unwrap();
                }
                writeln!(f, "  year = {},", self.0.published.year()).unwrap();
            }
            _ => {
                writeln!(f, "  year = {},", self.0.published.year()).unwrap();
            }
        };
        writeln!(f, "  doi = {},", BibStr::new(&self.0.doi)).unwrap();
        write!(f, "}}").unwrap();

        Ok(())
    }
}

struct BibStr<'a> {
    str: &'a str,
    verbatim: bool,
}

impl<'a> BibStr<'a> {
    fn new(str: &'a str) -> Self {
        BibStr {
            str,
            verbatim: false,
        }
    }

    fn verb(str: &'a str) -> Self {
        BibStr {
            str,
            verbatim: true,
        }
    }

    // Inspired by biblatex::resolve::is_escapable()
    fn should_escape(&self, c: char) -> bool {
        match c {
            '{' | '}' | '\\' => true,
            '~' | '^' | '#' | '&' | '%' | '$' | '_' if !self.verbatim => true,
            _ => false,
        }
    }
}

impl<'a> Display for BibStr<'a> {
    // Inspired by biblatex::ChunkExt::to_biblatex_string
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_char('{')?;
        if self.verbatim {
            f.write_char('{')?;
        }
        for c in self.str.chars() {
            if self.should_escape(c) {
                f.write_char('\\')?;
            }
            f.write_char(c)?;
        }
        f.write_char('}')?;
        if self.verbatim {
            f.write_char('}')?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plain() {
        let s = "hi there";
        assert_eq!(format!("{}", BibStr::new(s)), r"{hi there}");
    }

    #[test]
    fn test_verb() {
        let s = "hi there";
        assert_eq!(format!("{}", BibStr::verb(s)), r"{{hi there}}");
    }

    #[test]
    fn test_ampersand() {
        let s = "hi & there";
        assert_eq!(format!("{}", BibStr::new(s)), r"{hi \& there}");
    }

    #[test]
    fn test_ampersand_verb() {
        let s = "hi & there";
        assert_eq!(format!("{}", BibStr::verb(s)), r"{{hi & there}}");
    }
}
