use std::fs;
use std::path::{Path, PathBuf};

use voice_brief_desktop::store::{Store, data_root};

const TAKE: &str = r#"{"n":2,"at":2,"type":"TakeRecorded","take":"t1","stream":"s","text":"hello","wav":"/w.wav","ok":true}"#;

fn fresh_root(name: &str) -> PathBuf {
    let root: PathBuf = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).expect("root");
    root
}

fn session(root: &Path, project: &str, started: &str, pid: u32, extra: &str) {
    let dir: PathBuf = root.join(project).join("notes").join(started);
    fs::create_dir_all(&dir).expect("session dir");
    let start: String = format!(r#"{{"n":1,"at":1,"type":"SessionStarted","pid":{pid},"project":"/home/you/{project}"}}"#);
    fs::write(dir.join("thread.jsonl"), format!("{start}\n{extra}\n")).expect("thread");
}

const DEAD_PID: u32 = 4_000_000_000;

#[test]
fn data_root_prefers_xdg_data_home() {
    assert_eq!(data_root(Some("/x"), Some("/h")), Some(PathBuf::from("/x/voice-brief/projects")));
}

#[test]
fn data_root_falls_back_to_home_when_xdg_is_unset_or_empty() {
    assert_eq!((data_root(None, Some("/h")), data_root(Some(""), Some("/h"))), (Some(PathBuf::from("/h/.local/share/voice-brief/projects")), Some(PathBuf::from("/h/.local/share/voice-brief/projects"))));
}

#[test]
fn data_root_without_home_is_none() {
    assert_eq!(data_root(None, None), None);
}

#[test]
fn a_missing_root_has_no_sessions() {
    assert_eq!(Store::default().scan(Path::new("/nonexistent/voice-brief")), Vec::new());
}

#[test]
fn one_session_with_a_take_is_listed_with_its_project_name() {
    let root: PathBuf = fresh_root("one");
    session(&root, "-home-you-repo", "100", DEAD_PID, TAKE);
    let found = Store::default().scan(&root);
    assert_eq!(found.iter().map(|s| (s.started, s.project_name(), s.is_alive())).collect::<Vec<_>>(), vec![(100, "-home-you-repo", false)]);
}

#[test]
fn an_ended_session_without_takes_is_hidden_but_a_live_one_is_shown() {
    let root: PathBuf = fresh_root("many");
    session(&root, "a", "100", DEAD_PID, "");
    session(&root, "b", "200", std::process::id(), "");
    session(&root, "c", "300", DEAD_PID, TAKE);
    let mut found: Vec<u64> = Store::default().scan(&root).iter().map(|s| s.started).collect();
    found.sort_unstable();
    assert_eq!(found, vec![200, 300]);
}

#[test]
fn a_rescan_sees_a_thread_that_grew() {
    let root: PathBuf = fresh_root("grew");
    session(&root, "a", "100", DEAD_PID, TAKE);
    let mut store: Store = Store::default();
    let before: usize = store.scan(&root)[0].thread.takes.len();
    std::thread::sleep(std::time::Duration::from_millis(20));
    session(&root, "a", "100", DEAD_PID, &format!("{TAKE}\n{}", TAKE.replace("t1", "t2")));
    assert_eq!((before, store.scan(&root)[0].thread.takes.len()), (1, 2));
}

#[test]
fn a_sessions_open_streams_live_beside_its_notes() {
    let root: PathBuf = fresh_root("open-streams");
    session(&root, "-home-you-repo", "100", DEAD_PID, TAKE);
    assert_eq!(Store::default().scan(&root)[0].open_streams(), Some(root.join("-home-you-repo/open")));
}
