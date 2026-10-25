use std::{env, fs, process};

fn main() {
    let Some(path) = env::args().nth(1) else {
        eprintln!("usage: thread-ts <types file to rewrite>");
        process::exit(2);
    };
    let current: String = fs::read_to_string(&path).unwrap_or_default();
    if let Err(e) = fs::write(&path, vb_thread::splice(&current)) {
        eprintln!("thread-ts: {path}: {e}");
        process::exit(1);
    }
}
