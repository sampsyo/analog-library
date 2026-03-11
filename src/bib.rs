use crate::crossref;
use std::fmt::{Display, Write};

/// Format authors for BibTex.
fn bib_authors(auth: Vec<crossref::Author>) -> String {
    let mut out = String::new();
    let mut first = true;
    for a in auth.into_iter() {
        if !first {
            out.push_str(" and ");
        }
        first = false;

        out.push_str(&a.given);
        out.push(' ');
        out.push_str(&a.family);
    }
    out
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

pub fn bibtex(paper: crossref::Paper) -> String {
    // Citation keys like `lamport1978`. I'm sure we can do a lot better, but
    // this is better than nothing.
    let citekey = format!(
        "{}{}",
        paper.author[0].family.to_lowercase(),
        paper.published.year()
    );

    let type_ = BibType::from_crossref(&paper.type_);
    let authors = bib_authors(paper.author);

    let mut out = String::new();
    writeln!(out, "@{type_}{{{citekey},").unwrap();
    writeln!(out, "  title = {},", BibStr::verb(&paper.title)).unwrap();
    writeln!(out, "  author = {},", BibStr::new(&authors)).unwrap();
    match type_ {
        BibType::Article => {
            writeln!(out, "  journal = {},", BibStr::new(&paper.container_title)).unwrap();
            if let Some(volume) = paper.volume {
                writeln!(out, "  volume = {},", BibStr::new(&volume)).unwrap();
            }
            if let Some(issue) = paper.issue {
                writeln!(out, "  issue = {},", BibStr::new(&issue)).unwrap();
            }
            writeln!(out, "  year = {},", paper.published.year()).unwrap();
            if let Some(month) = paper.published.month() {
                writeln!(out, "  month = {},", month).unwrap();
            }
            if let Some(day) = paper.published.day() {
                writeln!(out, "  day = {},", day).unwrap();
            }
        }
        BibType::InProceedings => {
            if let Some(venue) = paper.event {
                writeln!(out, "  booktitle = {},", BibStr::new(&venue)).unwrap();
            }
            writeln!(out, "  year = {},", paper.published.year()).unwrap();
        }
        _ => {
            writeln!(out, "  year = {},", paper.published.year()).unwrap();
        }
    };
    writeln!(out, "  doi = {},", BibStr::new(&paper.doi)).unwrap();
    write!(out, "}}").unwrap();

    out
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
