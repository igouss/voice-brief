#!/usr/bin/env python3
import json, os, sys, time
from pathlib import Path

TMP = Path.home() / "tmp"
INBOX = TMP / "vb-proto-inbox.jsonl"
LOG = TMP / "vb-proto-hook.jsonl"
STATE = TMP / "vb-proto-hook-state.json"
MARK = "[voice] Injecting transcript"


def log(**data):
    with LOG.open("a") as f:
        f.write(json.dumps({"at": time.strftime("%Y-%m-%dT%H:%M:%S"), **data}, ensure_ascii=False) + "\n")


def debug_file(session):
    per_session = Path.home() / ".claude" / "debug" / f"{session}.txt"
    if session and per_session.exists():
        return per_session, None
    pid = os.getppid()
    while pid > 1:
        args = Path(f"/proc/{pid}/cmdline").read_bytes().split(b"\0")
        for i, a in enumerate(args):
            if a == b"--debug-file" and i + 1 < len(args):
                return Path(args[i + 1].decode()), pid
            if a.startswith(b"--debug-file="):
                return Path(a.split(b"=", 1)[1].decode()), pid
        pid = int(Path(f"/proc/{pid}/stat").read_text().rsplit(")", 1)[1].split()[1])
    return None, None


def voice_since_last_submit(path):
    state = json.loads(STATE.read_text()) if STATE.exists() else {}
    start = state.get(str(path), 0)
    size = path.stat().st_size if path.exists() else 0
    if size < start:
        start = 0
    new = ""
    if size > start:
        with path.open("rb") as f:
            f.seek(start)
            new = f.read(size - start).decode(errors="replace")
    state[str(path)] = size
    STATE.write_text(json.dumps(state))
    return [l for l in new.splitlines() if MARK in l]


def main():
    hook = json.load(sys.stdin)
    prompt = hook.get("prompt", "")
    path, pid = debug_file(hook.get("session_id"))
    if path is None:
        log(decision="unknown", why="no debug log: start claude with DEBUG=1", prompt=prompt[:80])
        return
    lines = voice_since_last_submit(path)
    dictated = bool(lines)
    log(decision="dictated" if dictated else "typed", debug_file=str(path), claude_pid=pid, voice_lines=lines,
        session=hook.get("session_id"), prompt=prompt)
    if not dictated or not prompt.strip():
        return
    with INBOX.open("a") as f:
        f.write(json.dumps({"at": time.strftime("%Y-%m-%dT%H:%M:%S"), "source": "claude-code-voice",
                            "session": hook.get("session_id"), "cwd": hook.get("cwd"), "text": prompt},
                           ensure_ascii=False) + "\n")


if __name__ == "__main__":
    main()
