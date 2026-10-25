#!/usr/bin/env python3
import json, os, sys, time
from pathlib import Path

TMP = Path.home() / "tmp"
INBOX = TMP / "vb-proto-inbox.jsonl"
LOG = TMP / "vb-proto-hook.jsonl"
STATE = TMP / "vb-proto-hook-state.json"
AUDIO = TMP / "vb-proto-audio"
INDEX = AUDIO / "index.jsonl"
OPEN = AUDIO / "open"
FIRST_SUBMIT_WINDOW_SECS = 600
WAIT_SECS = 3


def log(**data):
    with LOG.open("a") as f:
        f.write(json.dumps({"at": time.strftime("%Y-%m-%dT%H:%M:%S"), **data}, ensure_ascii=False) + "\n")


def normalize(text):
    return " ".join(text.replace("’", "'").replace("‘", "'").split()).lower()


def streams():
    if not INDEX.exists():
        return []
    return [json.loads(l) for l in INDEX.read_text().splitlines() if l.strip()]


def matching(prompt, used, since):
    p = normalize(prompt)
    return [s for s in streams()
            if s["name"] not in used and s["final"].strip() and s["end"] > since and normalize(s["final"]) in p]


def main():
    hook = json.load(sys.stdin)
    prompt = hook.get("prompt", "")
    session = hook.get("session_id") or ""
    state = json.loads(STATE.read_text()) if STATE.exists() else {}
    used = set(state.get("used", []))
    last_submit = state.setdefault("last_submit", {})
    since = last_submit.get(session, time.time() - FIRST_SUBMIT_WINDOW_SECS)
    last_submit[session] = time.time()
    STATE.write_text(json.dumps(state))
    found = matching(prompt, used, since)
    deadline = time.time() + WAIT_SECS
    while not found and OPEN.exists() and any(OPEN.iterdir()) and time.time() < deadline:
        time.sleep(0.2)
        found = matching(prompt, used, since)
    if not found:
        log(decision="typed", session=hook.get("session_id"), prompt=prompt)
        return
    found.sort(key=lambda s: s["start"])
    state["used"] = sorted(used | {s["name"] for s in found})
    STATE.write_text(json.dumps(state))
    dictated = " ".join(s["final"] for s in found)
    audio = {"segments": [{k: s[k] for k in ("name", "wav", "bytes", "chunks", "final", "closed_by_client", "endpoint")}
                          for s in found],
             "ok": all(s["closed_by_client"] and s["endpoint"] for s in found),
             "edited": normalize(dictated) != normalize(prompt)}
    log(decision="dictated", session=hook.get("session_id"), prompt=prompt, audio=audio)
    with INBOX.open("a") as f:
        f.write(json.dumps({"at": time.strftime("%Y-%m-%dT%H:%M:%S"), "source": "claude-code-voice",
                            "session": hook.get("session_id"), "cwd": hook.get("cwd"), "text": prompt,
                            "dictated": dictated, "audio": audio}, ensure_ascii=False) + "\n")
    if os.environ.get("VB_BRIEF"):
        print(json.dumps({"decision": "block",
                          "reason": "voice-brief: added to the open idea as a take; not sent to Claude."}))


if __name__ == "__main__":
    main()
