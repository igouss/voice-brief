use std::fs;
use std::path::Path;
use std::process::Command;
use std::sync::mpsc::{self, Sender};
use std::thread;
use std::time::Duration;

use serde::Deserialize;
use vb_talk::Mode;

pub fn voice_mode(layers: &[Option<String>]) -> Mode {
    let mode: Option<String> = layers.iter().flatten().find_map(|text| {
        let value: serde_json::Value = serde_json::from_str(text).ok()?;
        value.get("voice")?.get("mode")?.as_str().map(str::to_owned)
    });
    match mode.as_deref() {
        Some("tap") => Mode::Tap,
        Some(_) | None => Mode::Hold,
    }
}

pub fn settings_layers(project: &Path, home: Option<&Path>) -> Vec<Option<String>> {
    let mut paths: Vec<std::path::PathBuf> = vec![project.join(".claude/settings.local.json"), project.join(".claude/settings.json")];
    paths.extend(home.map(|h| h.join(".claude/settings.json")));
    paths.iter().map(|p| fs::read_to_string(p).ok()).collect()
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Client {
    pub address: String,
    pub pid: u32,
}

pub fn parse_clients(json: &str) -> Vec<Client> {
    serde_json::from_str(json).unwrap_or_default()
}

pub fn terminal_window(pid: u32, parent_of: impl Fn(u32) -> Option<u32>, clients: &[Client]) -> Option<String> {
    std::iter::successors(Some(pid), |&p| parent_of(p).filter(|&q| q > 1))
        .take(64)
        .find_map(|p| clients.iter().find(|c| c.pid == p))
        .map(|c| c.address.clone())
}

pub fn parent_pid(pid: u32) -> Option<u32> {
    let status: String = fs::read_to_string(format!("/proc/{pid}/status")).ok()?;
    status.lines().find_map(|l| l.strip_prefix("PPid:")).and_then(|v| v.trim().parse().ok())
}

pub fn is_recording(open_dir: &Path, pid: u32) -> bool {
    let wanted: String = pid.to_string();
    fs::read_dir(open_dir)
        .into_iter()
        .flatten()
        .flatten()
        .any(|e| fs::read_to_string(e.path()).is_ok_and(|text| text.trim() == wanted))
}

pub fn space_key(address: &str, down: bool) -> String {
    let state: &str = if down { "down" } else { "up" };
    format!("hl.dsp.send_key_state({{ mods = '', key = 'space', state = '{state}', window = 'address:{address}' }})")
}

fn dispatch(lua: &str) -> bool {
    Command::new("hyprctl").args(["dispatch", lua]).output().is_ok_and(|o| o.status.success())
}

fn clients() -> Vec<Client> {
    Command::new("hyprctl").args(["clients", "-j"]).output().map(|o| parse_clients(&String::from_utf8_lossy(&o.stdout))).unwrap_or_default()
}

#[derive(Debug)]
pub struct Talker {
    taps: Sender<u32>,
}

impl Talker {
    pub fn start() -> Self {
        let (taps, queue) = mpsc::channel::<u32>();
        thread::spawn(move || {
            for pid in queue {
                if let Some(address) = terminal_window(pid, parent_pid, &clients()) {
                    let pressed: bool = dispatch(&space_key(&address, true));
                    thread::sleep(Duration::from_millis(50));
                    let released: bool = dispatch(&space_key(&address, false));
                    if !(pressed && released) {
                        eprintln!("voice-brief: hyprctl could not send Space to {address}");
                    }
                }
            }
        });
        Self { taps }
    }

    pub fn tap(&self, pid: u32) {
        let _ = self.taps.send(pid);
    }
}

