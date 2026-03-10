use crate::core::{Context, Error, fetch_doi_json, render_paper};
use crate::view;
use axum::{
    Router,
    extract::{Path, State},
    http::{
        StatusCode,
        header::{ACCEPT, CONTENT_TYPE, HeaderMap, HeaderValue},
    },
    response::{IntoResponse, Response},
    routing::get,
};

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        dbg!(&self);
        let msg = self.to_string();
        let code = match self {
            Error::NotFound => StatusCode::NOT_FOUND,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };
        (code, msg).into_response()
    }
}

fn json_resp(json: &[u8]) -> Response {
    let buf = Vec::<u8>::from(json);
    (
        [(
            CONTENT_TYPE,
            HeaderValue::from_static(mime::APPLICATION_JSON.as_ref()),
        )],
        buf,
    )
        .into_response()
}

async fn show_paper(
    State(ctx): State<Context>,
    headers: HeaderMap,
    Path(doi): Path<String>,
) -> Result<impl IntoResponse, Error> {
    let paper_json = fetch_doi_json(&ctx, &doi).await?;
    match headers.get(ACCEPT).map(|x| x.as_bytes()) {
        Some(b"application/json") => Ok(json_resp(paper_json.as_ref())),
        _ => {
            let paper = serde_json::from_slice(paper_json.as_ref())?;
            Ok(render_paper(&ctx, paper).await?.into_response())
        }
    }
}

async fn show_home(headers: HeaderMap) -> maud::Markup {
    let host = match headers.get("Host") {
        Some(h) => h.to_str().unwrap_or("example.com"),
        None => "example.com",
    };
    view::home_page(host)
}

pub async fn serve(ctx: Context) {
    let app = Router::new()
        .route("/", get(show_home))
        .route("/doi/{*doi}", get(show_paper))
        .with_state(ctx);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8118").await.unwrap();
    eprintln!("listening on http://{}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}
