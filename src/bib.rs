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
    let mut entry = Entry::new("foo".to_string(), EntryType::InProceedings);
    entry.set_title(verbatim(paper.title()));
    entry.set_date(year(paper.published.year().try_into().unwrap()));
    entry.set_book_title(normal(paper.event.unwrap_or_else(|| "".to_string())));
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
    entry.to_bibtex_string().unwrap()
}
