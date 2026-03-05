mod crossref;
mod webcache;

use axum::{
    Router,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
};
use basset::assets;
use maud::{DOCTYPE, Markup, PreEscaped, html};

assets!(ASSETS, "assets", ["style.css"]);

#[derive(Debug, thiserror::Error)]
enum Error {
    #[error("failed loading paper data")]
    Fetch(#[from] webcache::Error),
    #[error("could not parse API response")]
    Parse(#[from] serde_json::Error),
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let body = self.to_string();
        (StatusCode::INTERNAL_SERVER_ERROR, body).into_response()
    }
}

/// Check whether a DOI is valid.
///
/// Matches the regex: [0-9/.]+
fn valid_doi(doi: &str) -> bool {
    if doi.is_empty() {
        return false;
    }
    for c in doi.bytes() {
        if !(c == b'/' || c == b'.' || c.is_ascii_digit()) {
            return false;
        }
    }
    true
}

async fn fetch_doi(db: &sled::Db, doi: &str) -> Result<Option<crossref::Paper>, Error> {
    if !valid_doi(doi) {
        return Ok(None);
    }
    let doi_url = format!("https://api.crossref.org/v1/works/{doi}/transform");
    if let Some(body) = webcache::fetch(db, &doi_url).await? {
        let paper = serde_json::from_slice(body.as_ref())?;
        Ok(Some(paper))
    } else {
        Ok(None)
    }
}

fn paper_page(paper: crossref::Paper, abstract_: Option<String>) -> Markup {
    #[cfg(not(debug_assertions))]
    let css = ASSETS.get("style.css").expect("asset must exist");

    #[cfg(debug_assertions)]
    let css = ASSETS.read("style.css").expect("asset must exist").unwrap();

    let title = paper.title();

    html! {
        (DOCTYPE)
        html {
            head {
                meta charset="utf-8";
                title { (title) };
                style { (PreEscaped(css)) };
            }
        }
        body {
            main {
                h1 { (title) };
                ul.authors {
                    @for author in paper.author {
                        li { (author.name()) }
                    }
                };
                @if let Some(abs) = abstract_ {
                    p.abstract { (abs) };
                } @else {
                    p.abstract.missing { "Abstract missing." };
                }
            }
        }
    }
}

/// Find an abstract for this paper.
///
/// Some papers in the Crossref database have several "identical" entries, with
/// different DOIs and different sets of metadata. When a paper is missing an
/// abstract, it is often the case that other identical entries *do* have an
/// abstract. So we first try the abstract we already have and, if it's missing,
/// try all the identical entries to see if they have one we can use.
async fn get_abstract(db: &sled::Db, paper: &crossref::Paper) -> Result<Option<String>, Error> {
    match &paper.abstract_ {
        Some(abs) => Ok(Some(abs.to_string())),
        None => {
            let mut out = None;
            for other_doi in paper.identical_dois() {
                if let Some(other_paper) = fetch_doi(db, &other_doi).await?
                    && let Some(abstract_) = other_paper.abstract_
                {
                    out = Some(abstract_.to_string());
                    break;
                }
            }
            Ok(out)
        }
    }
}

async fn show_paper(
    State(db): State<sled::Db>,
    Path(doi): Path<String>,
) -> Result<Markup, Response> {
    if let Some(paper) = fetch_doi(&db, &doi).await.map_err(|e| e.into_response())? {
        let abstract_ = get_abstract(&db, &paper)
            .await
            .map_err(|e| e.into_response())?;
        Ok(paper_page(paper, abstract_))
    } else {
        Err(StatusCode::NOT_FOUND.into_response())
    }
}

#[tokio::main]
async fn main() {
    let db = sled::open("cache.db").unwrap();

    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/doi/{*doi}", get(show_paper))
        .with_state(db);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8118").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
