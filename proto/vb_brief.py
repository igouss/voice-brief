#!/usr/bin/env python3
import json, os, readline, select, shutil, subprocess, sys, tempfile, time, traceback
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
CONTRACT = ROOT / "spike" / "contract"
RUN_SH = ROOT / "spike" / "isolation" / "run.sh"
FIELDS = [("weakest_part", "Weakest part"), ("force", "Force"), ("smallest_move", "Smallest move"),
          ("preserves", "Preserves"), ("would_reverse_it", "Would reverse it")]
INBOX = Path.home() / "tmp" / "vb-proto-inbox.jsonl"
WORK = Path(tempfile.mkdtemp(prefix="vb-brief-"))
LATEST = WORK.parent / "vb-brief-latest"
TRACE = (WORK / "trace.jsonl").open("a", buffering=1)
THREAD = (WORK / "thread.jsonl").open("a", buffering=1)
T0 = time.time()

thread = []
takes = {}
proposal = None
accepted = None
status = ""


def trace(kind, **data):
    TRACE.write(json.dumps({"t": round(time.time() - T0, 3), "kind": kind, **data}, ensure_ascii=False) + "\n")


def ev(etype, **data):
    e = {"n": len(thread) + 1, "type": etype, **data}
    thread.append(e)
    THREAD.write(json.dumps(e, ensure_ascii=False) + "\n")
    trace("event", event=e)
    return e


def run(cmd, **kw):
    t0 = time.time()
    r = subprocess.run(cmd, capture_output=True, text=True, **kw)
    trace("exec", cmd=[str(c) for c in cmd], rc=r.returncode, secs=round(time.time() - t0, 2),
          stdout=r.stdout[-4000:], stderr=r.stderr[-4000:])
    return r


def live():
    gone = {e["take"] for e in thread if e["type"] == "TakeDeleted"}
    return {t: x for t, x in takes.items() if t not in gone}


def rev():
    return sum(1 for e in thread if e["type"] == "Accepted")


def norm(s):
    return " ".join(s.replace("’", "'").replace("‘", "'").split())


def drain_inbox():
    global status
    if not INBOX.exists():
        return False
    taken = WORK / f"inbox-{time.time_ns()}.jsonl"
    INBOX.rename(taken)
    for line in taken.read_text().splitlines():
        if not line.strip():
            continue
        entry = json.loads(line)
        tid = f"t{len(takes) + 1}"
        takes[tid] = entry["text"]
        audio = entry.get("audio") or {}
        ev("TakeRecorded", take=tid, source=entry["source"], text=entry["text"], dictated=entry.get("dictated"),
           edited=audio.get("edited"), audio_ok=audio.get("ok"),
           wavs=[s["wav"] for s in audio.get("segments", []) if "wav" in s])
        status = f"{tid} arrived from Claude Code"
    return True


def transcript():
    return "\n\n".join(f"[{t}] {x}" for t, x in live().items())


def take_of(span):
    n = norm(span)
    return next((t for t, x in live().items() if n in norm(x)), "?")


def place():
    global proposal, status
    if not live():
        status = "no takes yet: dictate in the brief pane"
        return
    out = WORK / f"place-{len(thread) + 1}"
    out.mkdir()
    (out / "transcript.txt").write_text(transcript())
    r = run([CONTRACT / "target/debug/render", ROOT / "idea.md", CONTRACT / "prompt/system.md"])
    (out / "system.md").write_text(r.stdout)
    job = ev("JobSubmitted", base=rev(), ctx=sorted(live()), dir=str(out))
    print(f"\n  placing {', '.join(sorted(live()))} (isolated claude -p, about 15-40 s)...", flush=True)
    t0 = time.time()
    r = run([RUN_SH, "isolated", out / "system.md", out / "transcript.txt", out], stdin=subprocess.DEVNULL)
    if r.returncode != 0:
        status = f"model call failed (rc {r.returncode}); see {out}"
        ev("JobFailed", job=job["n"], rc=r.returncode)
        return
    v = run([CONTRACT / "target/debug/validate", "brief", out / "transcript.txt", out / "reply.txt"])
    secs = round(time.time() - t0, 1)
    if v.returncode != 0:
        status = f"reply rejected by the validator after {secs}s: {v.stdout.strip()[:300]}"
        ev("JobFailed", job=job["n"], violations=v.stdout.strip().splitlines())
        return
    events = json.loads(v.stdout)
    proposal = {"job": job["n"], "base": rev(), "ctx": sorted(live()),
                "claims": events[0]["fields"],
                "asked": {e["field"]: e["question"] for e in events[1:]}}
    ev("ProposalMade", job=job["n"], kind="place", secs=secs, claims=proposal["claims"], asked=proposal["asked"])
    status = f"proposal ready in {secs}s: [a]ccept or [x] reject; answer a question by dictating"


def fresh(p):
    return p["base"] == rev() and set(p["ctx"]) <= set(live())


def accept():
    global proposal, accepted, status
    if not proposal:
        status = "nothing to accept"
        return
    if not fresh(proposal):
        status = "stale: takes changed since this proposal; [p]lace again"
        trace("accept_refused", base=proposal["base"], rev=rev(), ctx=proposal["ctx"], live=sorted(live()))
        return
    ev("Accepted", job=proposal["job"])
    accepted, proposal = proposal, None
    status = f"accepted as revision {rev()}; [h]and off when it says what you mean"


def prompt_md(p):
    parts = []
    for key, title in FIELDS:
        claims = p["claims"].get(key, [])
        body = "\n".join(f"- {c['text']}" for c in claims) if claims else f"(open: {p['asked'].get(key, '')})"
        parts.append(f"## {title}\n\n{body}")
    return "\n\n".join(parts) + "\n"


def hand_off():
    global status
    if not accepted:
        status = "accept a proposal first"
        return
    text = prompt_md(accepted)
    path = WORK / "prompt.md"
    path.write_text(text)
    copied = shutil.which("wl-copy") and subprocess.run(["wl-copy"], input=text, text=True, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL).returncode == 0
    ev("HandedOff", target="clipboard" if copied else "file", revision=rev(), path=str(path))
    status = f"revision {rev()} {'copied to the clipboard and ' if copied else ''}written to {path}"


def edit_take():
    global status
    t = input("  take id: ").strip()
    if t not in live():
        status = f"no live take {t}"
        return
    readline.set_startup_hook(lambda: readline.insert_text(takes[t]))
    try:
        new = input("  fix: ").strip()
    finally:
        readline.set_startup_hook()
    if new and new != takes[t]:
        ev("TranscriptCorrected", take=t, before=takes[t], after=new)
        takes[t] = new
        status = f"{t} corrected; [p]lace again"


def delete_take():
    global status
    t = input("  take id: ").strip()
    if t in live():
        ev("TakeDeleted", take=t, reason=input("  reason: "))
        status = f"{t} deleted; [p]lace again"


def listen():
    t = input("  take id: ").strip()
    e = next((e for e in thread if e["type"] == "TakeRecorded" and e["take"] == t), {})
    for w in e.get("wavs", []):
        run(["pw-play", w])


def show():
    os.system("clear")
    print(f"voice-brief PROTOTYPE 2 (Claude Code mode)   revision {rev()}   log {WORK}")
    print("\nTAKES" + ("" if takes else "  (none yet: dictate in the brief pane)"))
    for t, x in takes.items():
        e = next(e for e in thread if e["type"] == "TakeRecorded" and e["take"] == t)
        flags = ["deleted" if t not in live() else "", "edited" if e.get("edited") else "",
                 "audio" if e.get("wavs") else "no audio"]
        print(f"  {t} [{', '.join(f for f in flags if f)}] {x[:150]}{'…' if len(x) > 150 else ''}")
    view, label = (proposal, "PROPOSAL") if proposal else (accepted, f"ACCEPTED revision {rev()}")
    if view:
        stale = proposal is not None and not fresh(proposal)
        print(f"\n{label} from {', '.join(view['ctx'])}" + ("   [STALE]" if stale else ""))
        for key, title in FIELDS:
            print(f"\n  {title}")
            for c in view["claims"].get(key, []):
                src = (f"said {', '.join(sorted({take_of(s) for s in c['said']}))}" if "said" in c
                       else f"ADDED: {c['added']}")
                print(f"    - {c['text']}\n        [{src}]")
            if key in view["asked"]:
                print(f"    ? {view['asked'][key]}")
    print(f"\n{status}")
    print("\n[p]lace  [a]ccept  [x]reject  [h]and off  [e]dit take  [d]elete take  [l]isten  [q]uit   (Enter refreshes)")
    print("> ", end="", flush=True)


def main():
    global proposal, status
    if LATEST.is_symlink() or not LATEST.exists():
        LATEST.unlink(missing_ok=True)
        LATEST.symlink_to(WORK)
    r = run(["cargo", "build", "-q", "--bins"], cwd=CONTRACT)
    if r.returncode != 0:
        print(r.stderr)
        return
    trace("start", work=str(WORK), pid=os.getpid())
    show()
    while True:
        ready, _, _ = select.select([sys.stdin], [], [], 1.0)
        if not ready:
            if drain_inbox():
                show()
            continue
        line = sys.stdin.readline()
        if not line:
            return
        k = line.strip()
        trace("input", key=k)
        drain_inbox()
        if k == "q":
            print(f"thread and trace in {WORK}")
            return
        elif k == "p":
            place()
        elif k == "a":
            accept()
        elif k == "x" and proposal:
            ev("Rejected", job=proposal["job"], reason=input("  reason: "))
            proposal = None
            status = "rejected"
        elif k == "h":
            hand_off()
        elif k == "e":
            edit_take()
        elif k == "d":
            delete_take()
        elif k == "l":
            listen()
        show()


if __name__ == "__main__":
    try:
        main()
    except BaseException as e:
        trace("crash", error=repr(e), tb=traceback.format_exc())
        raise
