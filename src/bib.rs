use std::fmt::{Display, Write};

use crate::crossref;
use biblatex::{
    Chunk, Chunks, Date, DateValue, Datetime, Entry, EntryType, PermissiveType, Person, Spanned,
};

fn verbatim(s: String) -> Chunks {
    vec![Spanned::zero(Chunk::Verbatim(s))]
}

fn normal(s: String) -> Chunks {
    vec![Spanned::zero(Chunk::Normal(s))]
}

fn year(y: i32) -> PermissiveType<Date> {
    PermissiveType::Typed(Date {
        value: DateValue::At(Datetime {
            year: y,
            month: None,
            day: None,
            time: None,
        }),
        uncertain: false,
        approximate: false,
    })
}

fn date(date: crossref::Date) -> PermissiveType<Date> {
    PermissiveType::Typed(Date {
        value: DateValue::At(Datetime {
            year: date.date_parts[0][0].try_into().unwrap(),
            month: date.date_parts[0].get(1).map(|&i| i.try_into().unwrap()),
            day: date.date_parts[0].get(2).map(|&i| i.try_into().unwrap()),
            time: None,
        }),
        uncertain: false,
        approximate: false,
    })
}

pub fn bibtex(paper: crossref::Paper) -> String {
    // Citation keys like `lamport1978`. I'm sure we can do a lot better, but
    // this is better than nothing.
    let citekey = format!(
        "{}{}",
        paper.author[0].family.to_lowercase(),
        paper.published.year()
    );

    // Map types to BibTeX types. Maybe someday we want to handle other kinds,
    // but the first two will do fine, falling back to `@misc`.
    let type_ = if paper.type_ == "journal-article" {
        EntryType::Article
    } else if paper.type_ == "proceedings-article" {
        EntryType::InProceedings
    } else {
        EntryType::Misc
    };

    // Always set title, author, and DOI.
    let mut entry = Entry::new(citekey, type_.clone());
    entry.set_title(verbatim(paper.title()));
    entry.set_author(
        paper
            .author
            .into_iter()
            .map(|a| Person {
                name: a.family,
                given_name: a.given,
                prefix: "".to_string(),
                suffix: "".to_string(),
            })
            .collect(),
    );
    entry.set_doi(paper.doi);

    // Type-specific fields.
    match type_ {
        EntryType::Article => {
            entry.set_journal(normal(paper.container_title));
            entry.set_volume(PermissiveType::Chunks(normal(
                paper.volume.unwrap_or_default(),
            )));
            entry.set_issue(normal(paper.issue.unwrap_or_default()));
            entry.set_date(date(paper.published));
        }
        EntryType::InProceedings => {
            entry.set_date(year(paper.published.year().try_into().unwrap()));
            entry.set_book_title(normal(paper.event.unwrap_or_default()));
        }
        _ => {
            entry.set_date(year(paper.published.year().try_into().unwrap()));
        }
    };

    entry.to_bibtex_string().unwrap()
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
