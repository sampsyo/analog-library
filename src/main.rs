use axum::{Router, extract::Path, routing::get};

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct DOIData {
    title: String,
}

async fn show_doi(Path(doi): Path<String>) -> String {
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

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/doi/{*doi}", get(show_doi));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8118").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
