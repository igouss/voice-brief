use std::collections::BTreeMap;

use crate::{Effect, Event, Pid, Talk, step};

const SESSION: Pid = 7;
const OTHER: Pid = 8;

pub const EXAMPLE_STATES: [Talk; 6] = [
    Talk::Idle,
    Talk::Starting { pid: SESSION },
    Talk::Recording { pid: SESSION },
    Talk::Releasing { pid: SESSION },
    Talk::Stopping { pid: SESSION },
    Talk::Settling { pid: SESSION },
];

pub const EXAMPLE_EVENTS: [Event; 9] = [
    Event::SpaceDown { target: Some(SESSION) },
    Event::SpaceDown { target: None },
    Event::SpaceUp,
    Event::FocusLost,
    Event::StreamOpened { pid: SESSION },
    Event::StreamClosed { pid: SESSION },
    Event::StreamOpened { pid: OTHER },
    Event::StreamClosed { pid: OTHER },
    Event::Quiet,
];

pub fn mermaid() -> String {
    let mut edges: BTreeMap<(usize, usize, bool), Vec<&'static str>> = BTreeMap::new();
    for (from, state) in EXAMPLE_STATES.iter().enumerate() {
        for event in EXAMPLE_EVENTS {
            let (next, effect): (Talk, Option<Effect>) = step(*state, event);
            if next == *state && effect.is_none() {
                continue;
            }
            let to: usize = EXAMPLE_STATES.iter().position(|s| *s == next).unwrap_or(from);
            edges.entry((from, to, effect.is_some())).or_default().push(event.name());
        }
    }
    let states: Vec<String> = EXAMPLE_STATES.iter().map(|s| format!("    {0}[{0}]", s.name())).collect();
    let transitions: Vec<String> = edges
        .into_iter()
        .enumerate()
        .map(|(i, ((from, to, taps), events))| {
            let label: String = events.join(", ");
            let node: String = if taps { format!("t{i}{{{{\"{label} / tap Space\"}}}}") } else { format!("t{i}([\"{label}\"])") };
            format!("    {} --> {node} --> {}", EXAMPLE_STATES[from].name(), EXAMPLE_STATES[to].name())
        })
        .collect();
    format!("flowchart TB\n    start((start)) --> Idle\n{}\n{}\n", states.join("\n"), transitions.join("\n"))
}
