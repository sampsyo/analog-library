//! Bindings for the Semantic Scholar API.

pub fn paper_url(doi: &str) -> String {
    format!("https://api.semanticscholar.org/graph/v1/paper/DOI:{doi}?fields=abstract")
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Paper {
    pub paper_id: String,
    pub title: String,
    pub authors: Vec<Author>,
    #[serde(rename = "abstract")]
    pub abstract_: Option<String>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Author {
    pub author_id: String,
    pub name: String,
}
