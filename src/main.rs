use axum::{
    Router,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
};
use basset::assets;
use maud::{DOCTYPE, Markup, PreEscaped, html};
use sled::transaction::TransactionResult;
use std::convert::Infallible;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const CACHE_EXPIRE: Duration = Duration::from_secs(60 * 60 * 24);

assets!(ASSETS, "assets", ["style.css"]);

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
struct DOIPaper {
    title: String,
    subtitle: Vec<String>,
    short_title: Vec<String>,
    author: Vec<DOIAuthor>,
    #[serde(rename = "type")]
    type_: String,
    #[serde(rename = "abstract")]
    abstract_: Option<String>,
    publisher: String,
    #[serde(rename = "URL")]
    url: String,

    container_title: String,
    page: String,
    volume: Option<String>,
    issue: Option<String>,

    issued: DOIDate,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
struct DOIDate {
    date_parts: Vec<(u32, u32, u32)>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
struct DOIAuthor {
    #[serde(rename = "ORCID")]
    orcid: Option<String>,
    given: String,
    family: String,
    sequence: String,
    affiliation: Vec<DOIAffiliation>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
struct DOIAffiliation {
    name: String,
}

impl DOIPaper {
    fn title(&self) -> String {
        let mut out = self.title.clone();
        for sub in self.subtitle.iter() {
            out.push_str(": ");
            out.push_str(sub);
        }
        out
    }
}

enum DullError {
    Fetch,
    Parse(serde_json::Error),
    Cache,
}

impl IntoResponse for DullError {
    fn into_response(self) -> Response {
        let body = match self {
            DullError::Fetch => "failed to retrieve data from doi.org".to_string(),
            DullError::Parse(e) => format!("could not parse doi.org response: {e}"),
            DullError::Cache => "error accessing cache".to_string(),
        };

        (StatusCode::INTERNAL_SERVER_ERROR, body).into_response()
    }
}

enum Cached<T> {
    /// We're caching actual contents.
    Valid(T),

    /// The cache entry is missing or expired.
    Invalid,

    /// The cache entry is present and fresh, but represents an error.
    Error,
}

/// Get the current cached URL contents.
fn cache_get(db: &sled::Db, url: &str) -> TransactionResult<Cached<sled::IVec>, Infallible> {
    let ts_key = format!("ts:{url}");

    db.transaction(|tx| {
        if let Some(ts_data) = tx.get(ts_key.as_bytes())? {
            let ts = u64::from_le_bytes(ts_data.as_ref().try_into().unwrap());
            let time = UNIX_EPOCH + Duration::from_secs(ts);

            // Is the cache entry expired?
            let age = SystemTime::now().duration_since(time).unwrap();
            if age > CACHE_EXPIRE {
                tx.remove(ts_key.as_bytes())?;
                tx.remove(url)?;
                Ok(Cached::Invalid)
            } else {
                match tx.get(url)? {
                    Some(body) => Ok(Cached::Valid(body)),
                    None => Ok(Cached::Error),
                }
            }
        } else {
            // Cold miss.
            Ok(Cached::Invalid)
        }
    })
}

/// Set the current cached contents of the URL.
fn cache_set(db: &sled::Db, url: &str, body: Cached<&[u8]>) -> TransactionResult<(), Infallible> {
    let ts_key = format!("ts:{url}");
    let ts_data = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
        .to_le_bytes();

    db.transaction(|tx| {
        tx.insert(ts_key.as_bytes(), &ts_data)?;
        match body {
            Cached::Valid(data) => tx.insert(url, data)?,
            Cached::Error => tx.remove(url)?, // Missing contents indicates error.
            Cached::Invalid => unimplemented!(),
        };
        Ok(())
    })
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

async fn fetch_doi(db: sled::Db, doi: &str) -> Result<Option<DOIPaper>, DullError> {
    if !valid_doi(doi) {
        return Ok(None);
    }
    let doi_url = format!("https://api.crossref.org/v1/works/{doi}/transform");

    match cache_get(&db, &doi_url).map_err(|_| DullError::Cache)? {
        Cached::Valid(body) => serde_json::from_slice(&body).map_err(DullError::Parse),
        Cached::Error => Ok(None),
        Cached::Invalid => {
            // Cache miss.
            let client = reqwest::Client::new();
            let res = client
                .get(&doi_url)
                .header("Accept", "application/json")
                .send()
                .await
                .map_err(|_| DullError::Fetch)?;
            if res.status() == StatusCode::OK {
                let body = res.bytes().await.map_err(|_| DullError::Fetch)?;
                cache_set(&db, &doi_url, Cached::Valid(body.as_ref()))
                    .map_err(|_| DullError::Cache)?;
                serde_json::from_slice(&body).map_err(DullError::Parse)
            } else {
                cache_set(&db, &doi_url, Cached::Error).map_err(|_| DullError::Cache)?;
                Ok(None)
            }
        }
    }
}

fn paper_page(paper: DOIPaper) -> Markup {
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
                p.abstract { (paper.abstract_.unwrap_or("".to_string())) };
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
