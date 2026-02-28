use axum::{
    Router,
    extract::{Path, State},
    routing::get,
};

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct DOIData {
    title: String,
}

async fn show_doi(State(ctx): State<Context>, Path(doi): Path<String>) -> String {
    // TODO validate DOI
    let doi_url = format!("https://doi.org/{doi}");

    // TODO cache results
    let client = reqwest::Client::new();
    let res: DOIData = client
        .get(doi_url)
        .header("Accept", "application/json")
        .send()
        .await
        .unwrap() // TODO
        .json()
        .await
        .unwrap(); // TODO
    dbg!(&res);

    format!("hi {}", res.title)
}

#[derive(Clone)]
struct Context {
    tmpls: minijinja::Environment<'static>,
}

#[tokio::main]
async fn main() {
    let ctx = Context {
        tmpls: minijinja::Environment::new(),
    };

    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/doi/{*doi}", get(show_doi))
        .with_state(ctx);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8118").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
