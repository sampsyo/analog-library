mod crossref;
mod jats;
mod serve;
mod view;
mod webcache;

use basset::assets;

assets!(ASSETS, "assets", ["style.css", "home.html"]);

fn user_agent() -> String {
    let base = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));
    match std::env::var("MAILTO") {
        Ok(email) if !email.is_empty() => format!("{base} (mailto:{email})"),
        _ => base.to_string(),
    }
}

#[derive(Clone)]
struct Context {
    db: sled::Db,
    client: reqwest::Client,
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
enum Error {
    #[error("failed loading paper data")]
    Fetch(#[from] webcache::Error),
    #[error("could not parse API response")]
    Parse(#[from] serde_json::Error),
    #[error("no paper entry found for this DOI")]
    NotFound,
}

/// Check whether a DOI is valid.
///
/// Matches the regex: [A-Za-z0-9/\.]+
fn valid_doi(doi: &str) -> bool {
    if doi.is_empty() {
        return false;
    }
    for c in doi.bytes() {
        if !(c == b'/' || c == b'.' || c.is_ascii_alphanumeric()) {
            return false;
        }
    }
    true
}

async fn fetch_doi_json(ctx: &Context, doi: &str) -> Result<sled::IVec, Error> {
    if !valid_doi(doi) {
        return Err(Error::NotFound);
    }
    let doi_url = format!("https://api.crossref.org/v1/works/{doi}/transform");
    webcache::fetch(&ctx.db, &ctx.client, &doi_url)
        .await?
        .ok_or(Error::NotFound)
}

async fn fetch_doi(ctx: &Context, doi: &str) -> Result<crossref::Paper, Error> {
    let json = fetch_doi_json(ctx, doi).await?;
    let paper = serde_json::from_slice(json.as_ref())?;
    Ok(paper)
}

fn main() {
    let ctx = Context::default();
    serve::serve(ctx);
}
