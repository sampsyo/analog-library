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

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("failed loading paper data")]
    Fetch(#[from] webcache::Error),
    #[error("could not parse API response")]
    Parse(#[from] serde_json::Error),
    #[error("no paper entry found for this DOI")]
    NotFound,
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

pub async fn fetch_doi_json(ctx: &Context, doi: &str) -> Result<sled::IVec, Error> {
    if !valid_doi(doi) {
        return Err(Error::NotFound);
    }
    let doi_url = format!("https://api.crossref.org/v1/works/{doi}/transform");
    webcache::fetch(&ctx.db, &ctx.client, &doi_url)
        .await?
        .ok_or(Error::NotFound)
}

pub async fn fetch_doi(ctx: &Context, doi: &str) -> Result<crossref::Paper, Error> {
    let json = fetch_doi_json(ctx, doi).await?;
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
async fn get_abstract(ctx: &Context, paper: &crossref::Paper) -> Result<Option<String>, Error> {
    match &paper.abstract_ {
        Some(abs) => Ok(Some(abs.to_string())),
        None => {
            let mut out = None;
            for other_doi in paper.identical_dois() {
                // TODO Maybe try to suppress "not found" errors when fetching other_paper?
                let other_paper = fetch_doi(ctx, &other_doi).await?;
                if let Some(abstract_) = other_paper.abstract_ {
                    out = Some(abstract_.to_string());
                    break;
                }
            }
            Ok(out)
        }
    }
}

pub async fn render_paper(ctx: &Context, paper: crossref::Paper) -> Result<Markup, Error> {
    let abstract_ = get_abstract(ctx, &paper).await?;
    Ok(view::paper_page(paper, abstract_))
}
