use vb_thread::{Body, FieldKey, Handoff, Placement, Source, Thread};

const REAL: &str = include_str!("fixtures/rejected-then-deleted.jsonl");

const TAKE_T1: &str = r#"{"n":2,"type":"TakeRecorded","at":2,"take":"t1","stream":"s1","text":"the sync module is weak","wav":"/w/s1.wav","ok":true}"#;
const TAKE_T2: &str = r#"{"n":3,"type":"TakeRecorded","at":3,"take":"t2","stream":"s2","text":"keep the desktop build","wav":"/w/s2.wav","ok":false}"#;
const SUBMIT: &str = r#"{"n":4,"type":"JobSubmitted","at":4,"base":0,"ctx":["t1","t2"],"dir":"/d"}"#;
const PROPOSE: &str = r#"{"n":5,"type":"ProposalMade","at":5,"kind":"place","secs":4.5,"claims":{"weakest_part":[{"id":"c1","text":"The sync module.","said":["the sync module"]}],"preserves":[{"id":"c2","text":"The desktop build.","added":"implied by t2"}]},"asked":{"force":"q1","smallest_move":"q2","would_reverse_it":"q3"}}"#;
const ACCEPT: &str = r#"{"n":6,"type":"Accepted","at":6,"revision":1}"#;
const HAND_OFF: &str = r#"{"n":7,"type":"HandedOff","at":7,"target":"clipboard","revision":1,"path":"/d/prompt.md"}"#;

fn lines(parts: &[&str]) -> String {
    parts.join("\n")
}

#[test]
fn no_events_is_the_empty_thread() {
    assert_eq!(Thread::replay(""), Thread::default());
}

#[test]
fn no_events_has_no_live_takes() {
    assert_eq!(Thread::replay("").live_takes(), 0);
}

#[test]
fn the_real_session_has_two_live_takes() {
    assert_eq!(Thread::replay(REAL).live_takes(), 2);
}

#[test]
fn each_field_carries_its_title() {
    assert_eq!(FieldKey::ALL.map(FieldKey::title), ["Weakest part", "Force", "Smallest move", "Preserves", "Would reverse it"]);
}

#[test]
fn one_recording_is_one_live_take() {
    let t: Thread = Thread::replay(TAKE_T1);
    assert_eq!((t.live_takes(), t.takes[0].text.as_str(), t.takes[0].clean), (1, "the sync module is weak", true));
}

#[test]
fn the_real_session_ends_idle_with_both_takes_recorded_again() {
    let t: Thread = Thread::replay(REAL);
    let ids: Vec<(&str, bool)> = t.takes.iter().map(|k| (k.id.as_str(), k.deleted)).collect();
    assert_eq!((t.pid, ids, t.placement, t.accepted, t.unreadable), (Some(1_071_994), vec![("t1", false), ("t2", false)], Placement::Idle, None, 0));
}

#[test]
fn the_real_proposal_asks_in_every_field() {
    let first_five: String = REAL.lines().take(5).collect::<Vec<&str>>().join("\n");
    let t: Thread = Thread::replay(&first_five);
    let Placement::Proposed(p) = t.placement else { panic!("not proposed: {:?}", t.placement) };
    let asked: Vec<bool> = p.fields.iter().map(|f| matches!(f.body, Body::Asked(_))).collect();
    assert_eq!((p.ctx, asked), (vec!["t1".to_owned(), "t2".to_owned()], vec![true; 5]));
}

#[test]
fn a_proposal_keeps_field_order_and_each_claims_source() {
    let t: Thread = Thread::replay(&lines(&[TAKE_T1, TAKE_T2, SUBMIT, PROPOSE]));
    let Placement::Proposed(p) = t.placement else { panic!("not proposed") };
    let shape: Vec<(FieldKey, Body)> = p.fields.into_iter().map(|f| (f.key, f.body)).collect();
    assert_eq!(
        shape.iter().map(|(k, _)| *k).collect::<Vec<FieldKey>>(),
        FieldKey::ALL.to_vec()
    );
    let Body::Claims(weakest) = &shape[0].1 else { panic!("weakest part has no claims") };
    let Body::Claims(preserves) = &shape[3].1 else { panic!("preserves has no claims") };
    assert_eq!(
        (&weakest[0].source, &preserves[0].source, &shape[1].1),
        (&Source::Said(vec!["the sync module".to_owned()]), &Source::Added("implied by t2".to_owned()), &Body::Asked("q1".to_owned()))
    );
}

#[test]
fn accepting_then_handing_off_records_the_revision_and_its_path() {
    let t: Thread = Thread::replay(&lines(&[TAKE_T1, SUBMIT, PROPOSE, ACCEPT, HAND_OFF]));
    let r = t.accepted.expect("accepted");
    assert_eq!((t.placement, r.number, r.handoff), (Placement::Idle, 1, Handoff::Done { path: "/d/prompt.md".to_owned() }));
}

#[test]
fn a_hand_off_of_another_revision_leaves_this_one_pending() {
    let other: &str = r#"{"n":8,"at":8,"type":"HandedOff","target":"clipboard","revision":2,"path":"/x"}"#;
    let t: Thread = Thread::replay(&lines(&[SUBMIT, PROPOSE, ACCEPT, other]));
    assert_eq!(t.accepted.map(|r| r.handoff), Some(Handoff::Pending));
}

#[test]
fn accepting_with_nothing_proposed_changes_nothing() {
    let t: Thread = Thread::replay(&lines(&[TAKE_T1, ACCEPT]));
    assert_eq!((t.placement, t.accepted), (Placement::Idle, None));
}

#[test]
fn rejecting_while_placing_keeps_placing() {
    let t: Thread = Thread::replay(&lines(&[SUBMIT, r#"{"n":5,"at":5,"type":"Rejected"}"#]));
    assert_eq!(t.placement, Placement::Placing { ctx: vec!["t1".to_owned(), "t2".to_owned()] });
}

#[test]
fn a_contract_failure_names_the_violations() {
    let failed: &str = r#"{"n":5,"at":5,"type":"JobFailed","violations":[{"violation":"span_not_in_transcript","detail":{}},{"violation":"missing_field","detail":"force"}]}"#;
    let t: Thread = Thread::replay(&lines(&[SUBMIT, failed]));
    assert_eq!(t.placement, Placement::Failed { reason: "the reply broke the contract: span_not_in_transcript, missing_field".to_owned() });
}

#[test]
fn a_model_without_a_reply_names_the_reason() {
    let t: Thread = Thread::replay(&lines(&[SUBMIT, r#"{"n":5,"at":5,"type":"JobFailed","reason":"timeout"}"#]));
    assert_eq!(t.placement, Placement::Failed { reason: "the model gave no reply (timeout)".to_owned() });
}

#[test]
fn sending_and_correcting_mark_the_take() {
    let sent: &str = r#"{"n":3,"at":3,"type":"TakeSent","take":"t1"}"#;
    let fixed: &str = r#"{"n":4,"at":4,"type":"TranscriptCorrected","take":"t1","before":"the sync module is weak","after":"the sync module is the weakest"}"#;
    let t: Thread = Thread::replay(&lines(&[TAKE_T1, sent, fixed]));
    assert_eq!((t.takes[0].sent, t.takes[0].corrected, t.takes[0].text.as_str()), (true, true, "the sync module is the weakest"));
}

#[test]
fn deleting_leaves_the_take_listed_but_not_live() {
    let t: Thread = Thread::replay(&lines(&[TAKE_T1, TAKE_T2, r#"{"n":4,"at":4,"type":"TakeDeleted","take":"t1"}"#]));
    assert_eq!((t.takes.len(), t.live_takes(), t.takes[0].deleted), (2, 1, true));
}

#[test]
fn an_event_type_from_a_newer_plugin_is_ignored() {
    let t: Thread = Thread::replay(&lines(&[TAKE_T1, r#"{"n":3,"at":3,"type":"SomethingNew","x":1}"#]));
    assert_eq!((t.live_takes(), t.unreadable), (1, 0));
}

#[test]
fn a_claim_with_both_sources_makes_its_line_unreadable() {
    let bad: &str = r#"{"n":5,"at":5,"type":"ProposalMade","kind":"place","secs":1,"claims":{"force":[{"id":"c1","text":"x","said":["a"],"added":"b"}]},"asked":{}}"#;
    let t: Thread = Thread::replay(&lines(&[SUBMIT, bad]));
    assert_eq!((t.unreadable, t.placement), (1, Placement::Placing { ctx: vec!["t1".to_owned(), "t2".to_owned()] }));
}

#[test]
fn a_torn_last_line_is_counted_not_fatal() {
    let t: Thread = Thread::replay(&format!("{TAKE_T1}\n{{\"type\":\"TakeRec"));
    assert_eq!((t.live_takes(), t.unreadable), (1, 1));
}

#[test]
fn a_proposal_naming_an_unknown_field_makes_its_line_unreadable() {
    let bad: &str = r#"{"n":5,"at":5,"type":"ProposalMade","kind":"place","secs":1,"claims":{},"asked":{"vibe":"q"}}"#;
    let t: Thread = Thread::replay(&lines(&[SUBMIT, bad]));
    assert_eq!(t.unreadable, 1);
}

#[test]
fn a_quote_is_traced_to_the_take_that_said_it() {
    let t: Thread = Thread::replay(&lines(&[TAKE_T1, TAKE_T2]));
    assert_eq!(t.quoted_from("keep  the desktop"), Some("t2"));
}

#[test]
fn a_quote_matching_no_take_has_no_source() {
    let t: Thread = Thread::replay(TAKE_T1);
    assert_eq!(t.quoted_from("never said"), None);
}

#[test]
fn an_empty_quote_has_no_source() {
    let t: Thread = Thread::replay(TAKE_T1);
    assert_eq!(t.quoted_from(" "), None);
}
