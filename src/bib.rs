use crate::crossref;
use std::fmt::{Display, Write};

/// The type of a BibTeX entry.
enum Type {
    Article,
    InProceedings,
    Misc,
}

impl Type {
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

impl Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::Article => f.write_str("article"),
            Type::InProceedings => f.write_str("inproceedings"),
            Type::Misc => f.write_str("misc"),
        }
    }
}

/// A BibTeX entry for a single paper.
pub struct Entry<'a>(pub &'a crossref::Paper);

impl<'a> Display for Entry<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let type_ = Type::from_crossref(&self.0.type_);
        let key = citekey(self.0);
        writeln!(f, "@{type_}{{{key},")?;
        write_pair(f, "title", BibStr::verb(&self.0.title))?;
        write_pair(f, "author", BibStr::new(Authors(&self.0.author)))?;
        match type_ {
            Type::Article => {
                write_str_opt(f, "journal", self.0.container_title.first())?;
                write_str_opt(f, "volume", self.0.volume.as_deref())?;
                write_str_opt(f, "issue", self.0.issue.as_deref())?;
                write_pair(f, "year", self.0.published.year())?;
                write_pair_opt(f, "month", self.0.published.month())?;
                write_pair_opt(f, "day", self.0.published.day())?;
            }
            Type::InProceedings => {
                write_str_opt(f, "booktitle", self.0.event_title())?;
                write_pair(f, "year", self.0.published.year())?;
            }
            _ => {
                write_pair(f, "year", self.0.published.year())?;
            }
        };
        write_str(f, "doi", &self.0.doi)?;
        write!(f, "}}")?;

        Ok(())
    }
}

/// Write a key/value pair in a BibTeX entry.
fn write_pair<T: Display>(
    f: &mut std::fmt::Formatter<'_>,
    key: &str,
    value: T,
) -> std::fmt::Result {
    writeln!(f, "  {} = {},", key, value)
}

/// Like `write_pair`, but only write anything if the value is Some.
fn write_pair_opt<T: Display>(
    f: &mut std::fmt::Formatter<'_>,
    key: &str,
    value: Option<T>,
) -> std::fmt::Result {
    if let Some(v) = value {
        write_pair(f, key, v)
    } else {
        Ok(())
    }
}

/// Like `write_pair`, but just for plain strings.
fn write_str(f: &mut std::fmt::Formatter<'_>, key: &str, value: &str) -> std::fmt::Result {
    writeln!(f, "  {} = {},", key, BibStr::new(value))
}

/// Like `write_str`, but just for optional strings.
fn write_str_opt(
    f: &mut std::fmt::Formatter<'_>,
    key: &str,
    value: Option<&str>,
) -> std::fmt::Result {
    if let Some(s) = value {
        write_str(f, key, s)
    } else {
        Ok(())
    }
}

/// A list of authors formatted for BibTeX.
struct Authors<'a>(&'a [crossref::Author]);

impl<'a> Display for Authors<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut first = true;
        for a in self.0.iter() {
            if !first {
                f.write_str(" and ")?;
            }
            first = false;

            if let Some(given) = &a.given {
                f.write_str(given)?;
                f.write_char(' ')?;
            }
            f.write_str(&a.family)?;
        }
        Ok(())
    }
}

/// A string formatted for a BibTeX value.
struct BibStr<T: Display> {
    value: T,
    verbatim: bool,
}

impl<T: Display> BibStr<T> {
    fn new(value: T) -> Self {
        BibStr {
            value,
            verbatim: false,
        }
    }

    fn verb(value: T) -> Self {
        BibStr {
            value,
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

impl<T: Display> Display for BibStr<T> {
    // Inspired by biblatex::ChunkExt::to_biblatex_string
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_char('{')?;
        if self.verbatim {
            f.write_char('{')?;
        }
        for c in self.value.to_string().chars() {
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

/// Generate a BibTeX citation key for a paper.
///
/// These citation keys currently look like `lamport1978`. I'm sure we can do a
/// lot better, but this is better than nothing.
fn citekey(paper: &crossref::Paper) -> String {
    let author: String = paper.author[0]
        .family
        .chars()
        .filter_map(|c| {
            if c.is_ascii_alphabetic() {
                Some(c.to_ascii_lowercase())
            } else {
                None
            }
        })
        .collect();
    format!("{}{}", author, paper.published.year())
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
