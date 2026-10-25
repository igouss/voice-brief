use super::*;

const TRANSCRIPT: &str = "The weakest part is the sync module. Every time I change the HTTP client,   the desktop build breaks. I don\u{2019}t want a new port layer.";

const ASKED: &str = r#"{"claims": [], "question": "What would show the move was wrong?"}"#;

fn reply_with(key: &str, field_json: &str) -> String {
    let parts: Vec<String> = FIELDS
        .iter()
        .map(|f: &Field| {
            let body: &str = if f.key() == key { field_json } else { ASKED };
            format!("\"{}\": {}", f.key(), body)
        })
        .collect();
    format!("{{\"fields\": {{{}}}}}", parts.join(", "))
}

fn said(id: &str, spans: &[&str]) -> String {
    let quoted: Vec<String> = spans.iter().map(|s: &&str| format!("{s:?}")).collect();
    format!(r#"{{"id": "{id}", "text": "t", "said": [{}]}}"#, quoted.join(", "))
}

fn violations(reply: &str) -> Vec<Violation> {
    validate(TRANSCRIPT, reply).err().unwrap_or_default()
}

fn claims_of(reply: &str, field: Field) -> usize {
    let placed: Placed = validate(TRANSCRIPT, reply).expect("valid");
    match placed.fields.get(&field) {
        Some(Placement::Claims(cs)) => cs.len(),
        Some(Placement::Asked(_)) | None => 0,
    }
}

#[test]
fn accepts_zero_claims_when_every_field_asks() {
    assert!(validate(TRANSCRIPT, &reply_with("force", ASKED)).is_ok());
}

#[test]
fn accepts_one_claim_with_one_span() {
    let field: String = format!("{{\"claims\": [{}]}}", said("c1", &["the sync module"]));
    assert_eq!(claims_of(&reply_with("weakest_part", &field), Field::WeakestPart), 1);
}

#[test]
fn accepts_many_claims_with_many_spans() {
    let field: String = format!(
        "{{\"claims\": [{}, {}]}}",
        said("c1", &["Every time I change the HTTP client", "the desktop build breaks"]),
        said("c2", &["the sync module"])
    );
    assert_eq!(claims_of(&reply_with("force", &field), Field::Force), 2);
}

#[test]
fn accepts_an_added_claim_with_a_reason() {
    let field: &str = r#"{"claims": [{"id": "c1", "text": "t", "added": "inferred from the build"}]}"#;
    assert_eq!(claims_of(&reply_with("preserves", field), Field::Preserves), 1);
}

#[test]
fn rejects_malformed_json() {
    assert!(matches!(violations("```json\n{}\n```").as_slice(), [Violation::Malformed(_)]));
}

#[test]
fn rejects_an_unknown_field() {
    let reply: String = reply_with("force", ASKED).replacen("{\"fields\": {", "{\"fields\": {\"extra\": {\"claims\": [], \"question\": \"q\"}, ", 1);
    assert_eq!(violations(&reply), vec![Violation::UnknownField("extra".into())]);
}

#[test]
fn rejects_a_missing_field() {
    let reply: String = reply_with("force", ASKED).replacen(&format!("\"preserves\": {ASKED}, "), "", 1);
    assert_eq!(violations(&reply), vec![Violation::MissingField(Field::Preserves)]);
}

#[test]
fn rejects_an_empty_field_without_a_question() {
    assert_eq!(violations(&reply_with("force", r#"{"claims": []}"#)), vec![Violation::EmptyWithoutQuestion(Field::Force)]);
}

#[test]
fn rejects_a_blank_question() {
    assert_eq!(
        violations(&reply_with("force", r#"{"claims": [], "question": "  "}"#)),
        vec![Violation::EmptyWithoutQuestion(Field::Force)]
    );
}

#[test]
fn rejects_claims_together_with_a_question() {
    let field: String = format!("{{\"claims\": [{}], \"question\": \"q\"}}", said("c1", &["the sync module"]));
    assert_eq!(violations(&reply_with("force", &field)), vec![Violation::ClaimsAndQuestion(Field::Force)]);
}

#[test]
fn rejects_a_claim_without_a_source() {
    let field: &str = r#"{"claims": [{"id": "c1", "text": "t"}]}"#;
    assert_eq!(
        violations(&reply_with("force", field)),
        vec![Violation::ClaimWithoutSource { field: Field::Force, claim: "c1".into() }]
    );
}

#[test]
fn rejects_a_claim_with_both_sources() {
    let field: &str = r#"{"claims": [{"id": "c1", "text": "t", "said": ["the sync module"], "added": "r"}]}"#;
    assert_eq!(
        violations(&reply_with("force", field)),
        vec![Violation::ClaimWithBothSources { field: Field::Force, claim: "c1".into() }]
    );
}

#[test]
fn rejects_a_claim_id_used_twice() {
    let field: String = format!("{{\"claims\": [{}, {}]}}", said("c1", &["the sync module"]), said("c1", &["the desktop build breaks"]));
    assert_eq!(violations(&reply_with("force", &field)), vec![Violation::DuplicateClaimId("c1".into())]);
}

#[test]
fn rejects_a_span_one_word_off_the_transcript() {
    let field: String = format!("{{\"claims\": [{}]}}", said("c1", &["the sync layer"]));
    assert_eq!(
        violations(&reply_with("weakest_part", &field)),
        vec![Violation::SpanNotInTranscript { field: Field::WeakestPart, claim: "c1".into(), span: "the sync layer".into() }]
    );
}

#[test]
fn rejects_an_empty_span() {
    let field: String = format!("{{\"claims\": [{}]}}", said("c1", &[" "]));
    assert_eq!(violations(&reply_with("force", &field)), vec![Violation::EmptySpan { field: Field::Force, claim: "c1".into() }]);
}

#[test]
fn rejects_a_span_that_differs_only_in_case() {
    let field: String = format!("{{\"claims\": [{}]}}", said("c1", &["The Sync Module"]));
    assert_eq!(violations(&reply_with("force", &field)).len(), 1);
}

#[test]
fn accepts_a_straight_apostrophe_for_a_curly_one() {
    let field: String = format!("{{\"claims\": [{}]}}", said("c1", &["I don't want a new port layer"]));
    assert_eq!(claims_of(&reply_with("force", &field), Field::Force), 1);
}

#[test]
fn accepts_a_span_whose_whitespace_differs() {
    let field: String = format!("{{\"claims\": [{}]}}", said("c1", &["HTTP client, the desktop\nbuild"]));
    assert_eq!(claims_of(&reply_with("force", &field), Field::Force), 1);
}

#[test]
fn events_ask_once_per_empty_field() {
    let field: String = format!("{{\"claims\": [{}]}}", said("c1", &["the sync module"]));
    let placed: Placed = validate(TRANSCRIPT, &reply_with("weakest_part", &field)).expect("valid");
    assert_eq!(events("t3", placed).len(), 5);
}

#[test]
fn events_start_with_a_place_proposal_for_the_take() {
    let placed: Placed = validate(TRANSCRIPT, &reply_with("force", ASKED)).expect("valid");
    assert!(matches!(events("t3", placed).first(), Some(Event::ProposalMade { kind: "place", take, .. }) if take == "t3"));
}

#[test]
fn format_table_is_read_from_idea_md() {
    let md: &str = "intro\n\n| Field | What it says |\n|---|---|\n| **Force** | two pressures |\n\nafter";
    assert_eq!(format_table(md), Some("| Field | What it says |\n|---|---|\n| **Force** | two pressures |"));
}

#[test]
fn format_table_is_none_without_the_table() {
    assert_eq!(format_table("no table here"), None);
}

#[test]
fn render_puts_the_definitions_in_the_template() {
    assert_eq!(render("defs:\n{{definitions}}\nend", "TABLE"), "defs:\nTABLE\nend");
}
