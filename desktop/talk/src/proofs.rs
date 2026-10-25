use crate::{EXAMPLE_EVENTS, EXAMPLE_STATES, Effect, Event, Pid, Talk, step};

const SESSION: Pid = 7;
const STEPS: usize = 8;
const QUEUE: usize = 10;

#[kani::proof]
fn every_state_is_drawn() {
    let state: Talk = kani::any();
    assert!(EXAMPLE_STATES.iter().any(|s| s.name() == state.name()));
}

#[kani::proof]
fn every_event_is_drawn() {
    let event: Event = kani::any();
    assert!(EXAMPLE_EVENTS.iter().any(|e| e.name() == event.name()));
}

fn pid_of(state: Talk) -> Option<Pid> {
    match state {
        Talk::Idle => None,
        Talk::Starting { pid } | Talk::Recording { pid } | Talk::Releasing { pid } | Talk::Stopping { pid } | Talk::Settling { pid } => Some(pid),
    }
}

#[kani::proof]
fn another_sessions_stream_changes_nothing() {
    let state: Talk = kani::any();
    let other: Pid = kani::any();
    kani::assume(pid_of(state) != Some(other));
    let opened: (Talk, Option<Effect>) = step(state, Event::StreamOpened { pid: other });
    let closed: (Talk, Option<Effect>) = step(state, Event::StreamClosed { pid: other });
    assert!(opened == (state, None) && closed == (state, None));
}

#[kani::proof]
fn a_tap_only_ever_goes_to_the_session_being_held() {
    let state: Talk = kani::any();
    let event: Event = kani::any();
    let (_, effect): (Talk, Option<Effect>) = step(state, event);
    let allowed: Option<Pid> = match event {
        Event::SpaceDown { target } if state == Talk::Idle => target,
        Event::SpaceDown { .. } | Event::SpaceUp | Event::FocusLost | Event::StreamOpened { .. } | Event::StreamClosed { .. } | Event::Quiet => pid_of(state),
    };
    assert!(effect.is_none() || effect.map(|Effect::Tap(p)| p) == allowed);
}

struct World {
    talk: Talk,
    recorder: bool,
    seen_streaming: bool,
    held: bool,
    queue: [Option<Event>; QUEUE],
}

impl World {
    fn push(&mut self, event: Event) {
        let slot: Option<&mut Option<Event>> = self.queue.iter_mut().find(|q| q.is_none());
        assert!(slot.is_some(), "the model's queue overflowed");
        if let Some(slot) = slot {
            *slot = Some(event);
        }
    }

    fn pop(&mut self) -> Option<Event> {
        let first: Option<Event> = self.queue[0];
        self.queue.rotate_left(1);
        self.queue[QUEUE - 1] = None;
        first
    }

    fn toggle_recorder(&mut self) {
        self.recorder = !self.recorder;
        let event: Event = if self.recorder { Event::StreamOpened { pid: SESSION } } else { Event::StreamClosed { pid: SESSION } };
        self.push(event);
    }

    fn feed(&mut self, event: Event) -> Option<Effect> {
        let (next, effect): (Talk, Option<Effect>) = step(self.talk, event);
        self.talk = next;
        if let Some(Effect::Tap(pid)) = effect {
            assert!(pid == SESSION);
            self.toggle_recorder();
        }
        effect
    }
}

fn deliver(w: &mut World) {
    if let Some(event) = w.pop() {
        w.seen_streaming = matches!(event, Event::StreamOpened { .. });
        w.feed(event);
    }
}

fn run() -> World {
    let mut w: World = World { talk: Talk::Idle, recorder: false, seen_streaming: false, held: false, queue: [None; QUEUE] };
    for _ in 0..STEPS {
        let action: u8 = kani::any();
        match action {
            0 if !w.held => {
                w.held = true;
                let can_record: bool = kani::any();
                let target: Option<Pid> = if can_record && !w.seen_streaming { Some(SESSION) } else { None };
                w.feed(Event::SpaceDown { target });
            }
            1 if w.held => {
                w.held = false;
                w.feed(Event::SpaceUp);
            }
            2 => {
                w.held = false;
                w.feed(Event::FocusLost);
            }
            3 => deliver(&mut w),
            4 if w.recorder => w.toggle_recorder(),
            5 => {
                w.feed(Event::StreamOpened { pid: SESSION + 1 });
            }
            6 if w.queue[0].is_none() => {
                w.feed(Event::Quiet);
            }
            _ => {}
        }
    }
    w
}

#[kani::proof]
#[kani::unwind(12)]
fn releasing_space_always_leaves_the_recorder_off() {
    let mut w: World = run();
    if !w.held {
        for _ in 0..QUEUE {
            deliver(&mut w);
        }
        assert!(w.queue[0].is_none());
        w.feed(Event::Quiet);
        assert!(!w.recorder, "the recorder was left on after Space was released");
        assert!(w.talk == Talk::Idle, "the window still thinks it holds the recorder");
    }
}

#[kani::proof]
fn the_window_taps_from_idle_only_when_space_is_pressed() {
    let event: Event = kani::any();
    let (_, effect): (Talk, Option<Effect>) = step(Talk::Idle, event);
    assert!(effect.is_none() || matches!(event, Event::SpaceDown { target: Some(_) }));
}
