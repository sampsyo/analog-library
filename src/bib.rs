use crate::crossref;
use biblatex::{Chunk, Chunks, Entry, EntryType, Spanned};

fn verbatim(s: String) -> Chunks {
    vec![Spanned::zero(Chunk::Verbatim(s))]
}

pub fn bibtex(paper: crossref::Paper) {
    let mut entry = Entry::new("foo".to_string(), EntryType::InProceedings);
    entry.set_title(verbatim(paper.title()));
    dbg!(&entry);
    println!("{}", entry.to_bibtex_string().unwrap());
}
