use serde_json::Value;

const CASES: &str = include_str!("../../../plugins/voice-notes/tests/contract-cases.ts");

fn cases() -> Value {
    serde_json::from_str(CASES.trim().trim_start_matches("export default ")).expect("cases parse")
}

fn outcome(transcript: &str, reply: &str) -> Value {
    match vb_contract::validate(transcript, reply) {
        Ok(_) => Value::String("ok".into()),
        Err(vs) => Value::Array(
            vs.iter()
                .map(|v| serde_json::to_value(v).expect("serialize")["violation"].clone())
                .collect(),
        ),
    }
}

fn pairs(all: &Value, pick: impl Fn(&Value) -> Value) -> Vec<(Value, Value)> {
    all["cases"].as_array().expect("cases").iter().map(|c| (c["name"].clone(), pick(c))).collect()
}

#[test]
fn the_rust_validator_agrees_with_every_shared_case() {
    let all: Value = cases();
    let transcript: String = all["transcript"].as_str().expect("transcript").to_string();
    let actual: Vec<(Value, Value)> = pairs(&all, |c| outcome(&transcript, c["reply"].as_str().expect("reply")));
    let expected: Vec<(Value, Value)> = pairs(&all, |c| c["expect"].clone());
    assert_eq!(actual, expected);
}
