#!/usr/bin/env python3
import json, os, sys, time
from pathlib import Path

TMP = Path.home() / "tmp"
INBOX = TMP / "vb-proto-inbox.jsonl"
LOG = TMP / "vb-proto-hook.jsonl"
STATE = TMP / "vb-proto-hook-state.json"
INDEX = TMP / "vb-proto-audio" / "index.jsonl"
OPEN = TMP / "vb-proto-audio" / "open"
WAIT_SECS = 3


def log(**data):
    with LOG.open("a") as f:
        f.write(json.dumps({"at": time.strftime("%Y-%m-%dT%H:%M:%S"), **data}, ensure_ascii=False) + "\n")


def normalize(text):
    return " ".join(text.replace("’", "'").replace("‘", "'").split()).lower()


def claude_pid():
    pid = os.getppid()
    while pid > 1:
        if Path(f"/proc/{pid}/comm").read_text().strip() == "claude":
            return pid
        pid = int(Path(f"/proc/{pid}/stat").read_text().rsplit(")", 1)[1].split()[1])
    return None


def matching(prompt, used, pid):
    p = normalize(prompt)
    rows = [json.loads(l) for l in INDEX.read_text().splitlines() if l.strip()] if INDEX.exists() else []
    return [s for s in rows if s["name"] not in used and s.get("pid") == pid and s["final"].strip()
            and normalize(s["final"]) in p]


def main():
    hook = json.load(sys.stdin)
    if not os.environ.get("VB_BRIEF"):
        return
    prompt = hook.get("prompt", "")
    pid = claude_pid()
    state = json.loads(STATE.read_text()) if STATE.exists() else {}
    used = set(state.get("used", []))
    found = matching(prompt, used, pid)
    deadline = time.time() + WAIT_SECS
    while not found and OPEN.exists() and any(OPEN.iterdir()) and time.time() < deadline:
        time.sleep(0.2)
        found = matching(prompt, used, pid)
    log(decision="sent" if found else "typed", session=hook.get("session_id"), claude_pid=pid, prompt=prompt,
        streams=[s["name"] for s in found])
    if not found:
        return
    state["used"] = sorted(used | {s["name"] for s in found})
    STATE.write_text(json.dumps(state))
    with INBOX.open("a") as f:
        f.write(json.dumps({"kind": "sent", "at": time.strftime("%Y-%m-%dT%H:%M:%S"),
                            "streams": [s["name"] for s in found], "prompt": prompt}, ensure_ascii=False) + "\n")


if __name__ == "__main__":
    main()
