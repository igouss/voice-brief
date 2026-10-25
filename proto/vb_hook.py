#!/usr/bin/env python3
import json, os, re, sys, time
from datetime import datetime
from pathlib import Path

TMP = Path.home() / "tmp"
INBOX = TMP / "vb-proto-inbox.jsonl"
LOG = TMP / "vb-proto-hook.jsonl"
STATE = TMP / "vb-proto-hook-state.json"
MARK = "[voice] Injecting transcript"
AUDIO = TMP / "vb-proto-audio"


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
    return new.splitlines()


def stamp(line):
    return datetime.fromisoformat(line[:24].replace("Z", "+00:00")).timestamp()


def recordings(lines):
    out, cur = [], None
    for l in lines:
        if "[voice] Starting recording session" in l:
            cur = {"start": stamp(l), "sent_bytes": 0}
        elif cur and "[voice_stream] Sending audio chunk:" in l:
            cur["sent_bytes"] += int(re.search(r"chunk: (\d+)", l).group(1))
        elif cur and "[voice] Recording stopped" in l:
            cur["end"] = stamp(l)
            out.append(cur)
            cur = None
    return out


def captured(seg):
    index = AUDIO / "index.jsonl"
    deadline = time.time() + 5
    while True:
        if index.exists():
            for l in index.read_text().splitlines():
                e = json.loads(l)
                if seg["start"] - 1 <= e["start"] <= seg["end"]:
                    return e
        if time.time() > deadline:
            return None
        time.sleep(0.2)


def save_audio(segments):
    if not segments:
        return {"error": "no recording span in the debug log"}
    takes = []
    for seg in segments:
        e = captured(seg)
        if e is None:
            takes.append({**seg, "error": "no voice stream captured by the proxy; was claude started with HTTPS_PROXY?"})
            continue
        takes.append({**seg, "wav": e["wav"], "captured_bytes": e["bytes"], "chunks": e["chunks"],
                      "exact": e["bytes"] == seg["sent_bytes"]})
    return {"segments": takes, "ok": all(t.get("exact") for t in takes)}


def main():
    hook = json.load(sys.stdin)
    prompt = hook.get("prompt", "")
    path, pid = debug_file(hook.get("session_id"))
    if path is None:
        log(decision="unknown", why="no debug log: start claude with DEBUG=1", prompt=prompt[:80])
        return
    new = voice_since_last_submit(path)
    lines = [l for l in new if MARK in l]
    dictated = bool(lines)
    log(decision="dictated" if dictated else "typed", debug_file=str(path), claude_pid=pid, voice_lines=lines,
        session=hook.get("session_id"), prompt=prompt)
    if not dictated or not prompt.strip():
        return
    audio = save_audio(recordings(new))
    log(decision="audio", audio=audio)
    with INBOX.open("a") as f:
        f.write(json.dumps({"at": time.strftime("%Y-%m-%dT%H:%M:%S"), "source": "claude-code-voice",
                            "session": hook.get("session_id"), "cwd": hook.get("cwd"), "text": prompt,
                            "audio": audio},
                           ensure_ascii=False) + "\n")


if __name__ == "__main__":
    main()
