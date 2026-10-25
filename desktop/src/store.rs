use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use vb_thread::Thread;

#[derive(Debug, Clone, PartialEq)]
pub struct Session {
    pub path: PathBuf,
    pub started: u64,
    pub modified: SystemTime,
    pub thread: Thread,
}

impl Session {
    pub fn project_name(&self) -> &str {
        let project: &str = self.thread.project.as_deref().unwrap_or("");
        project.rsplit('/').find(|s| !s.is_empty()).unwrap_or("unknown project")
    }

    pub fn is_alive(&self) -> bool {
        self.thread.pid.is_some_and(|pid| Path::new(&format!("/proc/{pid}")).exists())
    }
}

pub fn data_root(xdg_data_home: Option<&str>, home: Option<&str>) -> Option<PathBuf> {
    let base: PathBuf = match (xdg_data_home.filter(|s| !s.is_empty()), home) {
        (Some(xdg), _) => PathBuf::from(xdg),
        (None, Some(home)) => Path::new(home).join(".local/share"),
        (None, None) => return None,
    };
    Some(base.join("voice-brief/projects"))
}

fn thread_files(root: &Path) -> Vec<(PathBuf, u64)> {
    let projects = fs::read_dir(root).into_iter().flatten().flatten();
    projects
        .flat_map(|p| fs::read_dir(p.path().join("notes")).into_iter().flatten().flatten())
        .filter_map(|s| {
            let started: u64 = s.file_name().to_str()?.parse().ok()?;
            let path: PathBuf = s.path().join("thread.jsonl");
            path.is_file().then_some((path, started))
        })
        .collect()
}

#[derive(Debug, Default)]
pub struct Store {
    cache: HashMap<PathBuf, Session>,
}

impl Store {
    pub fn scan(&mut self, root: &Path) -> Vec<Session> {
        let mut sessions: Vec<Session> = thread_files(root)
            .into_iter()
            .filter_map(|(path, started)| {
                let modified: SystemTime = fs::metadata(&path).and_then(|m| m.modified()).ok()?;
                let cached: Option<&Session> = self.cache.get(&path).filter(|s| s.modified == modified);
                let session: Session = match cached {
                    Some(s) => s.clone(),
                    None => Session { thread: Thread::replay(&fs::read_to_string(&path).ok()?), path: path.clone(), started, modified },
                };
                self.cache.insert(path, session.clone());
                Some(session)
            })
            .filter(|s| !s.thread.takes.is_empty() || s.is_alive())
            .collect();
        sessions.sort_by_key(|s| std::cmp::Reverse(s.modified));
        sessions
    }
}
