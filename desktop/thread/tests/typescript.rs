use vb_thread::{splice, typescript};

const COMMITTED: &str = include_str!("../../../plugins/voice-notes/types/index.d.ts");

#[test]
fn the_plugins_types_hold_exactly_the_types_generated_from_the_wire_schema() {
    assert_eq!(splice(COMMITTED), COMMITTED, "regenerate: cargo run -q -p vb-thread --bin thread-ts -- ../plugins/voice-notes/types/index.d.ts");
}

#[test]
fn splicing_an_empty_file_gives_only_the_generated_types() {
    assert_eq!(splice(""), typescript());
}

#[test]
fn splicing_keeps_the_hand_written_rest_and_replaces_a_stale_block() {
    let stale: String = format!("{}\nexport type Old = 1\n// @end-generated\nexport type Mine = 2\n", vb_thread::GENERATED_START);
    assert_eq!(splice(&stale), format!("{}export type Mine = 2\n", typescript()));
}

#[test]
fn splicing_a_file_without_a_block_puts_the_types_in_front() {
    assert_eq!(splice("export type Mine = 2\n"), format!("{}export type Mine = 2\n", typescript()));
}
