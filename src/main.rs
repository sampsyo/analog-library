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

enum Error {
    Fetch(webcache::Error),
    Parse(serde_json::Error),
}

impl From<webcache::Error> for Error {
    fn from(err: webcache::Error) -> Self {
        Error::Fetch(err)
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error::Parse(err)
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let body = match self {
            Error::Fetch(webcache::Error::Web) => "failed to retrieve data from API".to_string(),
            Error::Fetch(webcache::Error::Cache) => "error accessing cache".to_string(),
            Error::Parse(e) => format!("could not parse API response: {e}"),
        };

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

async fn fetch_doi(db: sled::Db, doi: &str) -> Result<Option<crossref::Paper>, Error> {
    if !valid_doi(doi) {
        return Ok(None);
    }
    let doi_url = format!("https://api.crossref.org/v1/works/{doi}/transform");
    if let Some(body) = webcache::fetch(&db, &doi_url).await? {
        let paper = serde_json::from_slice(body.as_ref())?;
        Ok(Some(paper))
    } else {
        Ok(None)
    }
}

fn paper_page(paper: crossref::Paper) -> Markup {
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
                @if let Some(abs) = paper.abstract_ {
                    p.abstract { (abs) };
                } @else {
                    p.abstract.missing { "Abstract missing." };
                }
            }
        }
    }
}

async fn paper(State(db): State<sled::Db>, Path(doi): Path<String>) -> Result<Markup, Response> {
    match fetch_doi(db, &doi).await.map_err(|e| e.into_response())? {
        Some(doi_data) => Ok(paper_page(doi_data)),
        None => Err(StatusCode::NOT_FOUND.into_response()),
    }
}

#[tokio::main]
async fn main() {
    let db = sled::open("cache.db").unwrap();

    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/doi/{*doi}", get(paper))
        .with_state(db);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8118").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
