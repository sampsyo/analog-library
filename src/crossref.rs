//! Bindings for the Semantic Scholar API.

use std::{collections::HashMap, fmt::Display};

pub fn paper_url(doi: &str) -> String {
    format!("https://api.crossref.org/v1/works/{doi}/transform")
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Paper {
    pub title: String,
    pub subtitle: Vec<String>,
    pub short_title: Vec<String>,
    #[serde(default)]
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

    pub container_title: Strings,
    pub proceedings_subject: Option<String>,
    pub page: Option<String>,
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
    pub given: Option<String>,
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

/// A deserialization utility for fields that might be a string or might be an
/// array of strings.
#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
pub enum Strings {
    One(String),
    Many(Vec<String>),
}

impl Strings {
    pub fn first(&self) -> Option<&str> {
        match self {
            Strings::One(s) => Some(s),
            Strings::Many(v) => v.first().map(|s| s.as_ref()),
        }
    }
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

    pub fn is_acm(&self) -> bool {
        if let Some(url) = self.resource_url() {
            matches!(domain(url), Some(d) if d == "dl.acm.org" || d == "portal.acm.org")
        } else {
            false
        }
    }

    /// Get a direct link to the paper PDF, if one is known.
    pub fn pdf_url(&self) -> Option<String> {
        if self.is_acm() {
            Some(format!("https://dl.acm.org/doi/pdf/{}", self.doi))
        } else {
            None
        }
    }

    /// Get a link to a publisher page about the paper, if we have one.
    pub fn link_url(&self) -> Option<String> {
        if self.is_acm() {
            Some(format!("https://dl.acm.org/doi/{}", self.doi))
        } else {
            self.resource_url().map(str::to_string)
        }
    }

    pub fn human_type(&self) -> String {
        self.type_.replace("-", " ")
    }

    /// Assuming this is an article in some proceedings, guess a good name for
    /// the conference.
    pub fn event_title(&self) -> Option<&str> {
        if self.proceedings_subject.is_some() {
            // When the `proceedings-subject` field is present, then both it and
            // `event` seem to hold different fragments of the event name. Fall
            // back to the more generic container name.
            self.container_title.first()
        } else if let Some(s) = &self.event {
            // Otherwise, the event field usually has a friendlier name for the
            // conference than the generic container.
            Some(s)
        } else {
            None
        }
    }
}

impl Author {
    pub fn name(&self) -> String {
        match &self.given {
            Some(given) => format!("{} {}", given, self.family),
            None => self.family.to_string(), // RIP Arvind
        }
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

    pub fn iso(&self) -> String {
        match self.date_parts[0][..] {
            [y, m, d] => format!("{y:04}-{m:02}-{d:02}"),
            [y, m] => format!("{y:04}-{m:02}"),
            [y] => format!("{y:04}"),
            _ => String::new(),
        }
    }
}

impl Display for Date {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.date_parts[0][..] {
            [y, m, d] => write!(f, "{} {d}, {y}", month(m)),
            [y, m] => write!(f, "{} {y}", month(m)),
            [y] => write!(f, "{y}"),
            _ => Ok(()),
        }
    }
}

const MONTHS: [&str; 12] = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];

fn month(n: u32) -> &'static str {
    MONTHS.get((n - 1) as usize).copied().unwrap_or("?")
}

pub fn domain(url: &str) -> Option<String> {
    if let Ok(url) = url::Url::parse(url)
        && let Some(url::Host::Domain(dom)) = url.host()
    {
        Some(
            match dom.strip_prefix("www.") {
                Some(s) => s,
                None => dom,
            }
            .to_string(),
        )
    } else {
        None
    }
}
