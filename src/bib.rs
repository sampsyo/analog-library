use crate::crossref;
use biblatex::{
    Chunk, Chunks, Date, DateValue, Datetime, Entry, EntryType, PermissiveType, Spanned,
};

fn verbatim(s: String) -> Chunks {
    vec![Spanned::zero(Chunk::Verbatim(s))]
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
    entry.to_bibtex_string().unwrap()
}
