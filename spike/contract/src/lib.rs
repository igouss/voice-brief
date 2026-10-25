use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Field {
    WeakestPart,
    Force,
    SmallestMove,
    Preserves,
    WouldReverseIt,
}

pub const FIELDS: [Field; 5] = [
    Field::WeakestPart,
    Field::Force,
    Field::SmallestMove,
    Field::Preserves,
    Field::WouldReverseIt,
];

impl Field {
    pub fn key(self) -> &'static str {
        match self {
            Field::WeakestPart => "weakest_part",
            Field::Force => "force",
            Field::SmallestMove => "smallest_move",
            Field::Preserves => "preserves",
            Field::WouldReverseIt => "would_reverse_it",
        }
    }

    fn from_key(key: &str) -> Option<Field> {
        FIELDS.into_iter().find(|f: &Field| f.key() == key)
    }
}

#[derive(Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "violation", content = "detail", rename_all = "snake_case")]
pub enum Violation {
    Malformed(String),
    UnknownField(String),
    MissingField(Field),
    EmptyWithoutQuestion(Field),
    ClaimsAndQuestion(Field),
    ClaimWithoutSource { field: Field, claim: String },
    ClaimWithBothSources { field: Field, claim: String },
    DuplicateClaimId(String),
    EmptySpan { field: Field, claim: String },
    SpanNotInTranscript { field: Field, claim: String, span: String },
}

#[derive(Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Source {
    Said(Vec<String>),
    Added(String),
}

#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct Claim {
    pub id: String,
    pub text: String,
    #[serde(flatten)]
    pub source: Source,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Placement {
    Claims(Vec<Claim>),
    Asked(String),
}

#[derive(Debug, PartialEq, Eq)]
pub struct Placed {
    pub fields: BTreeMap<Field, Placement>,
}

#[derive(Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "type")]
pub enum Event {
    ProposalMade {
        kind: &'static str,
        take: String,
        fields: BTreeMap<Field, Vec<Claim>>,
    },
    Asked {
        field: Field,
        question: String,
    },
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WireOutput {
    fields: BTreeMap<String, WireField>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WireField {
    #[serde(default)]
    claims: Vec<WireClaim>,
    question: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WireClaim {
    id: String,
    text: String,
    #[serde(default)]
    said: Vec<String>,
    added: Option<String>,
}

pub fn normalize(text: &str) -> String {
    let straight: String = text
        .chars()
        .map(|c: char| match c {
            '\u{2018}' | '\u{2019}' => '\'',
            '\u{201C}' | '\u{201D}' => '"',
            other => other,
        })
        .collect();
    straight.split_whitespace().collect::<Vec<&str>>().join(" ")
}

pub fn validate(transcript: &str, reply: &str) -> Result<Placed, Vec<Violation>> {
    let wire: WireOutput = serde_json::from_str(reply.trim())
        .map_err(|e: serde_json::Error| vec![Violation::Malformed(e.to_string())])?;
    let haystack: String = normalize(transcript);
    let mut violations: Vec<Violation> = Vec::new();
    let mut fields: BTreeMap<Field, Placement> = BTreeMap::new();
    let mut ids: BTreeSet<String> = BTreeSet::new();
    for key in wire.fields.keys() {
        if Field::from_key(key).is_none() {
            violations.push(Violation::UnknownField(key.clone()));
        }
    }
    for field in FIELDS {
        match wire.fields.get(field.key()) {
            None => violations.push(Violation::MissingField(field)),
            Some(wf) => {
                if let Some(p) = place(field, wf, &haystack, &mut ids, &mut violations) {
                    fields.insert(field, p);
                }
            }
        }
    }
    if violations.is_empty() {
        Ok(Placed { fields })
    } else {
        Err(violations)
    }
}

fn place(
    field: Field,
    wf: &WireField,
    haystack: &str,
    ids: &mut BTreeSet<String>,
    violations: &mut Vec<Violation>,
) -> Option<Placement> {
    let question: Option<&str> = wf.question.as_deref().map(str::trim).filter(|q: &&str| !q.is_empty());
    match (wf.claims.is_empty(), question) {
        (true, None) => {
            violations.push(Violation::EmptyWithoutQuestion(field));
            None
        }
        (true, Some(q)) => Some(Placement::Asked(q.to_string())),
        (false, Some(_)) => {
            violations.push(Violation::ClaimsAndQuestion(field));
            None
        }
        (false, None) => {
            let claims: Vec<Claim> = wf
                .claims
                .iter()
                .filter_map(|c: &WireClaim| claim(field, c, haystack, ids, violations))
                .collect();
            Some(Placement::Claims(claims))
        }
    }
}

fn claim(
    field: Field,
    c: &WireClaim,
    haystack: &str,
    ids: &mut BTreeSet<String>,
    violations: &mut Vec<Violation>,
) -> Option<Claim> {
    if !ids.insert(c.id.clone()) {
        violations.push(Violation::DuplicateClaimId(c.id.clone()));
    }
    let source: Source = match (c.said.is_empty(), &c.added) {
        (true, None) => {
            violations.push(Violation::ClaimWithoutSource { field, claim: c.id.clone() });
            return None;
        }
        (false, Some(_)) => {
            violations.push(Violation::ClaimWithBothSources { field, claim: c.id.clone() });
            return None;
        }
        (true, Some(reason)) => Source::Added(reason.clone()),
        (false, None) => {
            for span in &c.said {
                let needle: String = normalize(span);
                if needle.is_empty() {
                    violations.push(Violation::EmptySpan { field, claim: c.id.clone() });
                } else if !haystack.contains(&needle) {
                    violations.push(Violation::SpanNotInTranscript {
                        field,
                        claim: c.id.clone(),
                        span: span.clone(),
                    });
                }
            }
            Source::Said(c.said.clone())
        }
    };
    Some(Claim { id: c.id.clone(), text: c.text.clone(), source })
}

pub fn events(take: &str, placed: Placed) -> Vec<Event> {
    let mut claims: BTreeMap<Field, Vec<Claim>> = BTreeMap::new();
    let mut asked: Vec<Event> = Vec::new();
    for (field, placement) in placed.fields {
        match placement {
            Placement::Claims(cs) => {
                claims.insert(field, cs);
            }
            Placement::Asked(question) => asked.push(Event::Asked { field, question }),
        }
    }
    let mut out: Vec<Event> = vec![Event::ProposalMade { kind: "place", take: take.to_string(), fields: claims }];
    out.extend(asked);
    out
}

pub fn format_table(idea_md: &str) -> Option<&str> {
    let start: usize = idea_md.find("| Field | What it says |")?;
    let rest: &str = &idea_md[start..];
    let end: usize = rest.find("\n\n").unwrap_or(rest.len());
    Some(&rest[..end])
}

pub fn render(template: &str, definitions: &str) -> String {
    template.replace("{{definitions}}", definitions)
}

#[cfg(test)]
mod tests;
