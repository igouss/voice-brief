use vb_talk::{Effect, Event, Mic, Mode, Talk, mic, step, target};

const SESSION: u32 = 7;

#[test]
fn pressing_space_taps_to_start() {
    assert_eq!(step(Talk::Idle, Event::SpaceDown { target: Some(SESSION) }), (Talk::Starting { pid: SESSION }, Some(Effect::Tap(SESSION))));
}

#[test]
fn pressing_space_for_a_session_that_cannot_record_does_nothing() {
    assert_eq!(step(Talk::Idle, Event::SpaceDown { target: None }), (Talk::Idle, None));
}

#[test]
fn releasing_while_recording_taps_to_stop() {
    assert_eq!(step(Talk::Recording { pid: SESSION }, Event::SpaceUp), (Talk::Stopping { pid: SESSION }, Some(Effect::Tap(SESSION))));
}

#[test]
fn releasing_before_the_stream_opens_stops_it_once_it_does() {
    let (releasing, none): (Talk, Option<Effect>) = step(Talk::Starting { pid: SESSION }, Event::SpaceUp);
    assert_eq!((releasing, none, step(releasing, Event::StreamOpened { pid: SESSION })), (Talk::Releasing { pid: SESSION }, None, (Talk::Stopping { pid: SESSION }, Some(Effect::Tap(SESSION)))));
}

#[test]
fn a_stream_that_reopens_while_settling_is_stopped_again() {
    assert_eq!(step(Talk::Settling { pid: SESSION }, Event::StreamOpened { pid: SESSION }), (Talk::Stopping { pid: SESSION }, Some(Effect::Tap(SESSION))));
}

#[test]
fn quiet_after_a_stop_ends_settling() {
    assert_eq!(step(Talk::Settling { pid: SESSION }, Event::Quiet), (Talk::Idle, None));
}

#[test]
fn losing_focus_while_recording_stops_like_a_release() {
    assert_eq!(step(Talk::Recording { pid: SESSION }, Event::FocusLost), step(Talk::Recording { pid: SESSION }, Event::SpaceUp));
}

#[test]
fn the_microphone_reads_ready_only_in_tap_mode_at_rest() {
    assert_eq!(
        (mic(true, Mode::Tap, Talk::Idle, false), mic(true, Mode::Hold, Talk::Idle, false), mic(false, Mode::Tap, Talk::Idle, false)),
        (Mic::Ready, Mic::NeedsTapMode, Mic::Ended)
    );
}

#[test]
fn an_open_stream_reads_recording_even_if_the_terminal_started_it() {
    assert_eq!(mic(true, Mode::Tap, Talk::Idle, true), Mic::Recording);
}

#[test]
fn only_a_ready_microphone_is_a_target() {
    assert_eq!((target(Some(SESSION), Mic::Ready), target(Some(SESSION), Mic::Recording), target(None, Mic::Ready)), (Some(SESSION), None, None));
}

#[test]
fn the_sessions_stream_opening_means_recording() {
    assert_eq!(step(Talk::Starting { pid: SESSION }, Event::StreamOpened { pid: SESSION }), (Talk::Recording { pid: SESSION }, None));
}

#[test]
fn the_sessions_stream_closing_after_a_stop_starts_settling() {
    assert_eq!(step(Talk::Stopping { pid: SESSION }, Event::StreamClosed { pid: SESSION }), (Talk::Settling { pid: SESSION }, None));
}

#[test]
fn the_recorder_stopping_by_itself_while_held_returns_to_idle() {
    assert_eq!(step(Talk::Recording { pid: SESSION }, Event::StreamClosed { pid: SESSION }), (Talk::Idle, None));
}

#[test]
fn another_sessions_stream_leaves_a_recording_alone() {
    assert_eq!(step(Talk::Recording { pid: SESSION }, Event::StreamClosed { pid: SESSION + 1 }), (Talk::Recording { pid: SESSION }, None));
}
