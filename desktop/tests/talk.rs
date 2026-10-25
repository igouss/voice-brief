use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use vb_talk::Mode;
use voice_brief_desktop::talk::{Client, is_recording, parent_pid, parse_clients, settings_layers, space_key, terminal_window, voice_mode};

const TAP: &str = r#"{"voice":{"enabled":true,"mode":"tap"}}"#;
const HOLD: &str = r#"{"voice":{"mode":"hold"}}"#;

fn layer(text: &str) -> Option<String> {
    Some(text.to_owned())
}

#[test]
fn no_settings_means_hold() {
    assert_eq!(voice_mode(&[None, None, None]), Mode::Hold);
}

#[test]
fn one_layer_saying_tap_means_tap() {
    assert_eq!(voice_mode(&[None, None, layer(TAP)]), Mode::Tap);
}

#[test]
fn the_nearest_layer_that_names_a_mode_wins() {
    assert_eq!((voice_mode(&[layer(HOLD), None, layer(TAP)]), voice_mode(&[layer("{}"), layer(TAP), layer(HOLD)])), (Mode::Hold, Mode::Tap));
}

#[test]
fn an_unreadable_layer_is_skipped() {
    assert_eq!(voice_mode(&[layer("{ not json"), layer(TAP)]), Mode::Tap);
}

fn clients() -> Vec<Client> {
    vec![Client { address: "0xaaa".to_owned(), pid: 100 }, Client { address: "0xbbb".to_owned(), pid: 200 }]
}

fn tree(pairs: &[(u32, u32)]) -> impl Fn(u32) -> Option<u32> {
    let map: HashMap<u32, u32> = pairs.iter().copied().collect();
    move |pid| map.get(&pid).copied()
}

#[test]
fn the_terminal_is_the_nearest_ancestor_with_a_window() {
    assert_eq!(terminal_window(300, tree(&[(300, 250), (250, 200), (200, 100)]), &clients()), Some("0xbbb".to_owned()));
}

#[test]
fn a_process_without_a_windowed_ancestor_has_no_terminal() {
    assert_eq!(terminal_window(300, tree(&[(300, 1)]), &clients()), None);
}

#[test]
fn no_windows_means_no_terminal() {
    assert_eq!(terminal_window(300, tree(&[(300, 200)]), &[]), None);
}

#[test]
fn hyprctl_clients_are_read_by_address_and_pid() {
    let json: &str = r#"[{"address":"0x5601","pid":42,"class":"com.mitchellh.ghostty"},{"address":"0x5602","pid":43}]"#;
    assert_eq!(parse_clients(json), vec![Client { address: "0x5601".to_owned(), pid: 42 }, Client { address: "0x5602".to_owned(), pid: 43 }]);
}

#[test]
fn unreadable_hyprctl_output_has_no_clients() {
    assert_eq!(parse_clients("hyprctl: no socket"), Vec::new());
}

#[test]
fn space_is_sent_to_the_window_by_address() {
    assert_eq!(
        (space_key("0x5601", true), space_key("0x5601", false)),
        (
            "hl.dsp.send_key_state({ mods = '', key = 'space', state = 'down', window = 'address:0x5601' })".to_owned(),
            "hl.dsp.send_key_state({ mods = '', key = 'space', state = 'up', window = 'address:0x5601' })".to_owned()
        )
    );
}

fn open_dir(name: &str, files: &[(&str, &str)]) -> PathBuf {
    let dir: PathBuf = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("open dir");
    files.iter().for_each(|(f, text)| fs::write(dir.join(f), text).expect("open file"));
    dir
}

#[test]
fn no_open_stream_is_not_recording() {
    assert!(!is_recording(&open_dir("open-none", &[]), 42));
}

#[test]
fn an_open_stream_of_this_process_is_recording() {
    assert!(is_recording(&open_dir("open-one", &[("s1", "42\n")]), 42));
}

#[test]
fn open_streams_of_other_processes_are_not_this_ones() {
    assert!(!is_recording(&open_dir("open-many", &[("s1", "7"), ("s2", ""), ("s3", "420")]), 42));
}


#[test]
fn the_walk_stops_before_init_even_if_init_owns_a_window() {
    let with_init: Vec<Client> = vec![Client { address: "0xinit".to_owned(), pid: 1 }];
    assert_eq!(terminal_window(300, tree(&[(300, 1)]), &with_init), None);
}

#[test]
fn a_process_s_parent_is_read_from_proc() {
    let mut child: std::process::Child = std::process::Command::new("sleep").arg("5").spawn().expect("spawn sleep");
    let parent: Option<u32> = parent_pid(child.id());
    let _ = child.kill();
    let _ = child.wait();
    assert_eq!(parent, Some(std::process::id()));
}

#[test]
fn settings_are_read_local_then_project_then_user() {
    let project: PathBuf = open_dir("settings-project", &[]);
    let home: PathBuf = open_dir("settings-home", &[]);
    fs::create_dir_all(project.join(".claude")).expect("project .claude");
    fs::create_dir_all(home.join(".claude")).expect("home .claude");
    fs::write(project.join(".claude/settings.local.json"), "local").expect("local");
    fs::write(home.join(".claude/settings.json"), "user").expect("user");
    assert_eq!(settings_layers(&project, Some(&home)), vec![Some("local".to_owned()), None, Some("user".to_owned())]);
}
