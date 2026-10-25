mod wire;

use std::collections::BTreeMap;

pub use wire::{Event, Failure, FieldKey, HandoffTarget, Line, ProposalKind, Violation, WireClaim, splice, typescript};
pub use wire::GENERATED_START;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Source {
    Said(Vec<String>),
    Added(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Claim {
    pub id: String,
    pub text: String,
    pub source: Source,
}

impl TryFrom<WireClaim> for Claim {
    type Error = String;

    fn try_from(w: WireClaim) -> Result<Self, Self::Error> {
        let source: Source = match (w.said.filter(|s| !s.is_empty()), w.added) {
            (Some(said), None) => Source::Said(said),
            (None, Some(reason)) => Source::Added(reason),
            (None, None) => return Err(format!("claim {} has no source", w.id)),
            (Some(_), Some(_)) => return Err(format!("claim {} has two sources", w.id)),
        };
        Ok(Self { id: w.id, text: w.text, source })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Take {
    pub id: String,
    pub text: String,
    pub wav: String,
    pub clean: bool,
    pub sent: bool,
    pub corrected: bool,
    pub deleted: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Body {
    Claims(Vec<Claim>),
    Asked(String),
    Missing,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Field {
    pub key: FieldKey,
    pub title: &'static str,
    pub body: Body,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Proposal {
    pub ctx: Vec<String>,
    pub secs: f64,
    pub fields: Vec<Field>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Placement {
    Idle,
    Placing { ctx: Vec<String> },
    Proposed(Proposal),
    Failed { reason: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Handoff {
    Pending,
    Done { path: String },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Revision {
    pub number: u32,
    pub proposal: Proposal,
    pub handoff: Handoff,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Thread {
    pub pid: Option<u32>,
    pub project: Option<String>,
    pub takes: Vec<Take>,
    pub placement: Placement,
    pub accepted: Option<Revision>,
    pub unreadable: usize,
}

impl Default for Thread {
    fn default() -> Self {
        Self {
            pid: None,
            project: None,
            takes: Vec::new(),
            placement: Placement::Idle,
            accepted: None,
            unreadable: 0,
        }
    }
}

fn fields_of(
    mut claims: BTreeMap<FieldKey, Vec<WireClaim>>,
    mut asked: BTreeMap<FieldKey, String>,
) -> Result<Vec<Field>, String> {
    FieldKey::ALL
        .iter()
        .map(|&key| {
            let placed: Vec<Claim> = claims.remove(&key).unwrap_or_default().into_iter().map(Claim::try_from).collect::<Result<_, _>>()?;
            let body: Body = match (placed.is_empty(), asked.remove(&key)) {
                (false, _) => Body::Claims(placed),
                (true, Some(question)) => Body::Asked(question),
                (true, None) => Body::Missing,
            };
            Ok(Field { key, title: key.title(), body })
        })
        .collect()
}

fn failure(f: Failure) -> String {
    match f {
        Failure::Contract { violations } => {
            let kinds: Vec<String> = violations.into_iter().map(|v| v.violation).collect();
            format!("the reply broke the contract: {}", kinds.join(", "))
        }
        Failure::NoReply { reason } => format!("the model gave no reply ({reason})"),
        Failure::Threw { error } => error,
    }
}

fn normalize(text: &str) -> String {
    let straight: String = text.replace(['‘', '’'], "'").replace(['“', '”'], "\"");
    straight.split_whitespace().collect::<Vec<&str>>().join(" ")
}

impl Thread {
    pub fn replay(text: &str) -> Self {
        text.lines()
            .filter(|line| !line.trim().is_empty())
            .fold(Self::default(), |thread, line| match serde_json::from_str::<Line>(line) {
                Ok(line) => thread.step(line.event),
                Err(_) => Self { unreadable: thread.unreadable + 1, ..thread },
            })
    }

    fn take_mut(&mut self, id: &str) -> Option<&mut Take> {
        self.takes.iter_mut().find(|t| t.id == id)
    }

    pub fn step(mut self, event: Event) -> Self {
        match event {
            Event::SessionStarted { pid, project, .. } => {
                self.pid = Some(pid);
                self.project = Some(project);
            }
            Event::SetUp { .. } | Event::Unknown => {}
            Event::TakeRecorded { take, text, wav, ok, .. } => {
                let fresh: Take = Take { id: take, text, wav, clean: ok, sent: false, corrected: false, deleted: false };
                match self.take_mut(&fresh.id) {
                    Some(existing) => *existing = fresh,
                    None => self.takes.push(fresh),
                }
            }
            Event::TakeSent { take } => {
                if let Some(t) = self.take_mut(&take) {
                    t.sent = true;
                }
            }
            Event::TakeDeleted { take } => {
                if let Some(t) = self.take_mut(&take) {
                    t.deleted = true;
                }
            }
            Event::TranscriptCorrected { take, after, .. } => {
                if let Some(t) = self.take_mut(&take) {
                    t.text = after;
                    t.corrected = true;
                }
            }
            Event::JobSubmitted { ctx, .. } => self.placement = Placement::Placing { ctx },
            Event::JobFailed { failure: f } => self.placement = Placement::Failed { reason: failure(f) },
            Event::ProposalMade { secs, claims, asked, .. } => {
                let Ok(fields) = fields_of(claims, asked) else {
                    self.unreadable += 1;
                    return self;
                };
                let ctx: Vec<String> = match self.placement {
                    Placement::Placing { ctx } => ctx,
                    Placement::Idle | Placement::Proposed(_) | Placement::Failed { .. } => Vec::new(),
                };
                self.placement = Placement::Proposed(Proposal { ctx, secs, fields });
            }
            Event::Accepted { revision } => {
                self.placement = match self.placement {
                    Placement::Proposed(proposal) => {
                        self.accepted = Some(Revision { number: revision, proposal, handoff: Handoff::Pending });
                        Placement::Idle
                    }
                    other @ (Placement::Idle | Placement::Placing { .. } | Placement::Failed { .. }) => other,
                };
            }
            Event::Rejected {} => {
                self.placement = match self.placement {
                    Placement::Proposed(_) => Placement::Idle,
                    other @ (Placement::Idle | Placement::Placing { .. } | Placement::Failed { .. }) => other,
                };
            }
            Event::HandedOff { revision, path, .. } => {
                if let Some(r) = self.accepted.as_mut().filter(|r| r.number == revision) {
                    r.handoff = Handoff::Done { path };
                }
            }
        }
        self
    }

    pub fn quoted_from(&self, span: &str) -> Option<&str> {
        let needle: String = normalize(span);
        self.takes.iter().find(|t| !needle.is_empty() && normalize(&t.text).contains(&needle)).map(|t| t.id.as_str())
    }

    pub fn live_takes(&self) -> usize {
        self.takes.iter().filter(|t| !t.deleted).count()
    }
}
