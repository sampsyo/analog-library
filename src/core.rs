use crate::{crossref, view, webcache};
use basset::assets;
use maud::Markup;

assets!(ASSETS, "assets", ["style.css", "home.html"]);

fn user_agent() -> String {
    let base = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));
    match std::env::var("MAILTO") {
        Ok(email) if !email.is_empty() => format!("{base} (mailto:{email})"),
        _ => base.to_string(),
    }
}

/// Check whether a DOI is valid.
///
/// Matches the regex: 10\.[0-9]+/[A-Za-z0-9\.-_;()/]+
/// As suggested by Crossref staff:
/// https://community.crossref.org/t/question-about-characters-in-doi-suffixes/3867/2
fn valid_doi(doi: &str) -> bool {
    if !doi.starts_with("10.") {
        return false;
    }
    for c in doi.bytes().skip(3) {
        if !(c.is_ascii_alphanumeric()
            || c == b'.'
            || c == b'-'
            || c == b'_'
            || c == b';'
            || c == b'('
            || c == b')'
            || c == b'/')
        {
            return false;
        }
    }
    true
}

pub fn join<T: AsRef<str>>(ss: impl Iterator<Item = T>, sep: &str) -> String {
    let mut out = String::new();
    let mut first = true;
    for s in ss {
        if first {
            first = false;
        } else {
            out.push_str(sep);
        }
        out.push_str(s.as_ref());
    }
    out
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("failed loading paper data")]
    Fetch(#[from] webcache::Error),
    #[error("could not parse API response")]
    Parse(#[from] serde_json::Error),
    #[error("no paper entry found for this DOI")]
    NotFound(String),
}

#[derive(Clone)]
pub struct Context {
    pub db: sled::Db,
    pub client: reqwest::Client,
}

impl Default for Context {
    fn default() -> Self {
        let db = sled::open("cache.db").unwrap();
        let client = reqwest::Client::builder()
            .user_agent(user_agent())
            .build()
            .unwrap();
        Self { db, client }
    }
}

impl Context {
    pub async fn fetch_doi_json(&self, doi: &str) -> Result<sled::IVec, Error> {
        if !valid_doi(doi) {
            return Err(Error::NotFound(doi.to_string()));
        }
        let doi_url = format!("https://api.crossref.org/v1/works/{doi}/transform");
        webcache::fetch(&self.db, &self.client, &doi_url)
            .await?
            .ok_or(Error::NotFound(doi.to_string()))
    }

    pub async fn fetch_doi(&self, doi: &str) -> Result<crossref::Paper, Error> {
        let json = self.fetch_doi_json(doi).await?;
        let paper = serde_json::from_slice(json.as_ref())?;
        Ok(paper)
    }

    /// Find an abstract for this paper.
    ///
    /// Some papers in the Crossref database have several "identical" entries, with
    /// different DOIs and different sets of metadata. When a paper is missing an
    /// abstract, it is often the case that other identical entries *do* have an
    /// abstract. So we first try the abstract we already have and, if it's missing,
    /// try all the identical entries to see if they have one we can use.
    pub async fn get_abstract(&self, paper: &crossref::Paper) -> Result<Option<String>, Error> {
        match &paper.abstract_ {
            Some(abs) => Ok(Some(abs.to_string())),
            None => {
                let mut out = None;
                for other_doi in paper.identical_dois() {
                    // TODO Maybe try to suppress "not found" errors when fetching other_paper?
                    let other_paper = self.fetch_doi(&other_doi).await?;
                    if let Some(abstract_) = other_paper.abstract_ {
                        out = Some(abstract_.to_string());
                        break;
                    }
                }
                Ok(out)
            }
        }
    }

    pub async fn render_paper(&self, paper: crossref::Paper) -> Result<Markup, Error> {
        let abstract_ = self.get_abstract(&paper).await?;
        Ok(view::paper(paper, abstract_))
    }

    pub fn dump_cache(&self) -> Result<(), Error> {
        for entry in webcache::cache_scan(&self.db) {
            let (url, time, json) = entry?;
            match serde_json::from_slice::<crossref::Paper>(json.as_ref()) {
                Ok(paper) => println!("{} {} {}", time, &paper.doi, paper.title()),
                Err(_) => println!(
                    "{} deserialization error: {}",
                    time,
                    String::from_utf8_lossy(&url)
                ),
            }
        }
        Ok(())
    }
}
