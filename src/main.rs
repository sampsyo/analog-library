use axum::{
    Router,
    extract::Path,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
};
use basset::assets;
use maud::{DOCTYPE, Markup, PreEscaped, html};

assets!(ASSETS, "assets", ["style.css"]);

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
struct DOIData {
    title: String,
    author: Vec<DOIAuthor>,
    #[serde(rename = "type")]
    type_: String,
    #[serde(rename = "abstract")]
    abstract_: String,
    publisher: String,
    #[serde(rename = "URL")]
    url: String,

    container_title: String,
    page: String,
    volume: String,
    issue: String,

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
    orcid: String,
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

enum MyError {
    Fetch,
    Parse(serde_json::Error),
}

impl IntoResponse for MyError {
    fn into_response(self) -> Response {
        let body = match self {
            MyError::Fetch => "failed to retrieve data from doi.org".to_string(),
            MyError::Parse(e) => format!("could not parse doi.org response: {e}"),
        };

        // it's often easiest to implement `IntoResponse` by calling other implementations
        (StatusCode::INTERNAL_SERVER_ERROR, body).into_response()
    }
}

async fn fetch_doi(doi: &str) -> Result<DOIData, MyError> {
    // TODO validate DOI
    let doi_url = format!("https://doi.org/{doi}");

    // TODO cache results
    let client = reqwest::Client::new();
    let body = client
        .get(doi_url)
        .header("Accept", "application/json")
        .send()
        .await
        .map_err(|_| MyError::Fetch)?
        .bytes()
        .await
        .map_err(|_| MyError::Fetch)?;

    serde_json::from_slice(&body).map_err(|e| MyError::Parse(e))
}

fn paper_page(paper: DOIData) -> Markup {
    #[cfg(not(debug_assertions))]
    let css = ASSETS.get("style.css").expect("asset must exist");

    #[cfg(debug_assertions)]
    let css = ASSETS.read("style.css").expect("asset must exist").unwrap();

    html! {
        (DOCTYPE)
        html {
            head {
                meta charset="utf-8";
                title { (paper.title) };
                style { (PreEscaped(css)) };
            }
        }
        body {
            main {
                h1 { (paper.title) };
                p.abstract { (paper.abstract_) };
            }
        }
    }
}

async fn paper(Path(doi): Path<String>) -> Result<Markup, MyError> {
    let doi_data = fetch_doi(&doi).await?;
    Ok(paper_page(doi_data))
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/doi/{*doi}", get(paper));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8118").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
