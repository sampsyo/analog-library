#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Paper {
    pub title: String,
    pub subtitle: Vec<String>,
    pub short_title: Vec<String>,
    pub author: Vec<Author>,
    #[serde(rename = "type")]
    pub type_: String,
    #[serde(rename = "abstract")]
    pub abstract_: Option<String>,
    pub publisher: String,
    #[serde(rename = "URL")]
    pub url: String,

    pub container_title: String,
    pub page: String,
    pub volume: Option<String>,
    pub issue: Option<String>,

    pub issued: Date,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Date {
    pub date_parts: Vec<(u32, u32, u32)>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Author {
    #[serde(rename = "ORCID")]
    pub orcid: Option<String>,
    given: String,
    pub family: String,
    sequence: String,
    pub affiliation: Vec<Affiliation>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Affiliation {
    pub name: String,
}

impl Paper {
    pub fn title(&self) -> String {
        let mut out = self.title.clone();
        for sub in self.subtitle.iter() {
            out.push_str(": ");
            out.push_str(sub);
        }
        out
    }
}

impl Author {
    pub fn name(&self) -> String {
        // TODO Use the `sequence` field.
        format!("{} {}", self.given, self.family)
    }
}
