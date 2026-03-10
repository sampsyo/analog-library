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
                paper.volume.unwrap_or_else(|| "".to_string()),
            )));
            entry.set_issue(normal(paper.issue.unwrap_or_else(|| "".to_string())));
        }
        EntryType::InProceedings => {
            entry.set_date(year(paper.published.year().try_into().unwrap()));
            entry.set_book_title(normal(paper.event.unwrap_or_else(|| "".to_string())));
        }
        _ => {
            entry.set_date(year(paper.published.year().try_into().unwrap()));
        }
    };

    entry.to_bibtex_string().unwrap()
}
