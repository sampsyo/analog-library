use crate::{crossref, jats, ss, view, webcache};
use basset::assets;
use maud::Markup;

// Load or embed static assets. The `RSRC` array contains the files that we will
// also serve under the `/rsrc/` directory.
assets!(
    ASSETS,
    "assets",
    ["style.css", "home.html", "bookmarklet.js", "userscript.js"]
);
pub const RSRC: &[(&str, mime::Mime)] = &[
    ("bookmarklet.js", mime::APPLICATION_JAVASCRIPT),
    ("userscript.js", mime::APPLICATION_JAVASCRIPT),
];

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

/// An abstract that is either encoded in JATS XML, as plain text, or missing altogether.
pub enum Abstract {
    Jats(String),
    Text(String),
    Missing,
}

impl Abstract {
    pub fn text(self) -> Option<String> {
        match self {
            Abstract::Jats(j) => match jats::to_text(&j) {
                Ok(t) => Some(t),
                Err(_) => Some(j),
            },
            Abstract::Text(t) => Some(t),
            Abstract::Missing => None,
        }
    }
}

/// The data sources for DOI data.
pub enum Source {
    Crossref,
    SemanticScholar,
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
    // Make an API request for the data for a DOI.
    pub async fn fetch_doi(&self, doi: &str, source: Source) -> Result<sled::IVec, Error> {
        if !valid_doi(doi) {
            return Err(Error::NotFound(doi.to_string()));
        }
        let url = match source {
            Source::Crossref => crossref::paper_url(doi),
            Source::SemanticScholar => ss::paper_url(doi),
        };
        webcache::fetch(&self.db, &self.client, &url)
            .await?
            .ok_or(Error::NotFound(doi.to_string()))
    }

    /// Get a paper from the Crossref API by its DOI.
    pub async fn crossref_paper(&self, doi: &str) -> Result<crossref::Paper, Error> {
        let json = self.fetch_doi(doi, Source::Crossref).await?;
        let paper = serde_json::from_slice(json.as_ref())?;
        Ok(paper)
    }

    /// Get a paper from the Semantic Scholar API by its DOI.
    pub async fn ss_paper(&self, doi: &str) -> Result<ss::Paper, Error> {
        let json = self.fetch_doi(doi, Source::SemanticScholar).await?;
        let paper = serde_json::from_slice(json.as_ref())?;
        Ok(paper)
    }

    /// Find an abstract for this paper, possibly by making additional API requests.
    pub async fn get_abstract(&self, paper: &crossref::Paper) -> Result<Abstract, Error> {
        match &paper.abstract_ {
            Some(abs) => Ok(Abstract::Jats(abs.to_string())),
            None => {
                // Some papers in the Crossref database have several "identical"
                // entries, with different DOIs and different sets of metadata.
                // When a paper is missing an abstract, it is often the case
                // that other identical entries *do* have an abstract.
                for other_doi in paper.identical_dois() {
                    match self.crossref_paper(&other_doi).await {
                        Ok(other_paper) => {
                            if let Some(abstract_) = other_paper.abstract_ {
                                return Ok(Abstract::Jats(abstract_.to_string()));
                            }
                        }
                        Err(Error::NotFound(_)) => continue,
                        Err(e) => return Err(e),
                    };
                }

                // Next, try the Semantic Scholar API.
                match self.ss_paper(&paper.doi).await {
                    Ok(ss_paper) => {
                        if let Some(abstract_) = ss_paper.abstract_ {
                            return Ok(Abstract::Text(abstract_));
                        }
                    }
                    Err(Error::NotFound(_)) => (),
                    Err(e) => return Err(e),
                };

                Ok(Abstract::Missing)
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
