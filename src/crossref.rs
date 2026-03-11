use std::collections::HashMap;

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
    pub issued: Date,
    pub published: Date,
    pub relation: HashMap<String, Vec<Relation>>,
    pub resource: HashMap<String, Resource>,
    #[serde(rename = "DOI")]
    pub doi: String,

    pub container_title: String,
    pub page: String,
    pub volume: Option<String>,
    pub issue: Option<String>,
    pub event: Option<String>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Date {
    pub date_parts: Vec<Vec<u32>>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Author {
    #[serde(rename = "ORCID")]
    pub orcid: Option<String>,
    pub given: String,
    pub family: String,
    pub sequence: String,
    pub affiliation: Vec<Affiliation>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Affiliation {
    pub name: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Relation {
    pub id_type: String,
    pub id: String,
    pub asserted_by: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Resource {
    #[serde(rename = "URL")]
    pub url: String,
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

    pub fn identical_dois(&self) -> Vec<String> {
        if let Some(rels) = self.relation.get("is-identical-to") {
            rels.iter().map(|r| r.id.clone()).collect()
        } else {
            vec![]
        }
    }

    pub fn resource_url(&self) -> Option<&str> {
        self.resource.get("primary").map(|r| r.url.as_ref())
    }

    pub fn domain(&self) -> Option<String> {
        if let Some(url) = self.resource_url()
            && let Ok(url) = url::Url::parse(url)
            && let Some(url::Host::Domain(dom)) = url.host()
        {
            Some(dom.to_string())
        } else {
            None
        }
    }

    pub fn is_acm(&self) -> bool {
        matches!(self.domain(), Some(d) if d == "dl.acm.org")
    }

    pub fn pdf_url(&self) -> Option<String> {
        if self.is_acm() {
            Some(format!("https://dl.acm.org/doi/pdf/{}", self.doi))
        } else {
            None
        }
    }

    pub fn human_type(&self) -> String {
        self.type_.replace("-", " ")
    }
}

impl Author {
    pub fn name(&self) -> String {
        // TODO Use the `sequence` field.
        format!("{} {}", self.given, self.family)
    }
}

impl Date {
    pub fn year(&self) -> u32 {
        self.date_parts[0][0]
    }

    pub fn month(&self) -> Option<u32> {
        self.date_parts[0].get(1).cloned()
    }

    pub fn day(&self) -> Option<u32> {
        self.date_parts[0].get(2).cloned()
    }
}
