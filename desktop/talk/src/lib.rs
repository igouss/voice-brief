mod diagram;
#[cfg(kani)]
mod proofs;

pub use diagram::{EXAMPLE_EVENTS, EXAMPLE_STATES, mermaid};

pub type Pid = u32;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(kani, derive(kani::Arbitrary))]
pub enum Talk {
    Idle,
    Starting { pid: Pid },
    Recording { pid: Pid },
    Releasing { pid: Pid },
    Stopping { pid: Pid },
    Settling { pid: Pid },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(kani, derive(kani::Arbitrary))]
pub enum Event {
    SpaceDown { target: Option<Pid> },
    SpaceUp,
    FocusLost,
    StreamOpened { pid: Pid },
    StreamClosed { pid: Pid },
    Quiet,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Effect {
    Tap(Pid),
}

pub fn step(state: Talk, event: Event) -> (Talk, Option<Effect>) {
    match (state, event) {
        (Talk::Idle, Event::SpaceDown { target: Some(pid) }) => (Talk::Starting { pid }, Some(Effect::Tap(pid))),
        (Talk::Starting { pid }, Event::StreamOpened { pid: opened }) if opened == pid => (Talk::Recording { pid }, None),
        (Talk::Starting { pid }, Event::SpaceUp | Event::FocusLost) => (Talk::Releasing { pid }, None),
        (Talk::Recording { pid }, Event::SpaceUp | Event::FocusLost) => (Talk::Stopping { pid }, Some(Effect::Tap(pid))),
        (Talk::Releasing { pid }, Event::StreamOpened { pid: opened }) if opened == pid => (Talk::Stopping { pid }, Some(Effect::Tap(pid))),
        (Talk::Stopping { pid }, Event::StreamClosed { pid: closed }) if closed == pid => (Talk::Settling { pid }, None),
        (Talk::Settling { pid }, Event::StreamOpened { pid: opened }) if opened == pid => (Talk::Stopping { pid }, Some(Effect::Tap(pid))),
        (Talk::Settling { .. }, Event::Quiet) => (Talk::Idle, None),
        (Talk::Starting { pid } | Talk::Recording { pid } | Talk::Releasing { pid }, Event::StreamClosed { pid: closed }) if closed == pid => {
            (Talk::Idle, None)
        }
        (
            Talk::Idle | Talk::Starting { .. } | Talk::Recording { .. } | Talk::Releasing { .. } | Talk::Stopping { .. } | Talk::Settling { .. },
            Event::SpaceDown { .. } | Event::SpaceUp | Event::FocusLost | Event::StreamOpened { .. } | Event::StreamClosed { .. } | Event::Quiet,
        ) => (state, None),
    }
}

impl Talk {
    pub const fn name(self) -> &'static str {
        match self {
            Self::Idle => "Idle",
            Self::Starting { .. } => "Starting",
            Self::Recording { .. } => "Recording",
            Self::Releasing { .. } => "Releasing",
            Self::Stopping { .. } => "Stopping",
            Self::Settling { .. } => "Settling",
        }
    }
}

impl Event {
    pub const fn name(self) -> &'static str {
        match self {
            Self::SpaceDown { target: Some(_) } => "Space down",
            Self::SpaceDown { target: None } => "Space down (cannot record)",
            Self::SpaceUp => "Space up",
            Self::FocusLost => "focus lost",
            Self::StreamOpened { .. } => "opened",
            Self::StreamClosed { .. } => "closed",
            Self::Quiet => "quiet 1 s",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Tap,
    Hold,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mic {
    Ended,
    NeedsTapMode,
    Ready,
    Starting,
    Recording,
    Stopping,
}

pub const fn mic(is_alive: bool, mode: Mode, talk: Talk, is_streaming: bool) -> Mic {
    match (is_alive, mode, talk, is_streaming) {
        (false, Mode::Tap | Mode::Hold, Talk::Idle | Talk::Starting { .. } | Talk::Recording { .. } | Talk::Releasing { .. } | Talk::Stopping { .. } | Talk::Settling { .. }, true | false) => Mic::Ended,
        (true, Mode::Hold, Talk::Idle | Talk::Starting { .. } | Talk::Recording { .. } | Talk::Releasing { .. } | Talk::Stopping { .. } | Talk::Settling { .. }, true | false) => Mic::NeedsTapMode,
        (true, Mode::Tap, Talk::Stopping { .. } | Talk::Releasing { .. } | Talk::Settling { .. }, true | false) => Mic::Stopping,
        (true, Mode::Tap, Talk::Idle | Talk::Starting { .. } | Talk::Recording { .. }, true) => Mic::Recording,
        (true, Mode::Tap, Talk::Starting { .. } | Talk::Recording { .. }, false) => Mic::Starting,
        (true, Mode::Tap, Talk::Idle, false) => Mic::Ready,
    }
}

pub const fn target(pid: Option<Pid>, mic: Mic) -> Option<Pid> {
    match mic {
        Mic::Ready => pid,
        Mic::Ended | Mic::NeedsTapMode | Mic::Starting | Mic::Recording | Mic::Stopping => None,
    }
}
