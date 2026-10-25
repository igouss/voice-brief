#!/usr/bin/env python3
import json, os, re, readline, signal, subprocess, sys, tempfile, time, traceback
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
FIELDS = ["Weakest part", "Force", "Smallest move", "Preserves", "Would reverse it"]
MODEL = os.environ.get("VB_MODEL", "claude-opus-5-5")
WORK = Path(tempfile.mkdtemp(prefix="vb-proto-"))
LATEST = WORK.parent / "vb-proto-latest"
TRACE = (WORK / "trace.jsonl").open("a", buffering=1)
THREAD = (WORK / "thread.jsonl").open("a", buffering=1)
T0 = time.time()
INBOX = Path.home() / "tmp" / "vb-proto-inbox.jsonl"

thread = []
takes = {}
fields = {f: "" for f in FIELDS}
proposal = None


def trace(kind, **data):
    TRACE.write(json.dumps({"t": round(time.time() - T0, 3), "kind": kind, **data}, ensure_ascii=False) + "\n")


def ask(prompt):
    try:
        answer = input(prompt)
    except EOFError:
        trace("input", prompt=prompt, eof=True)
        raise
    trace("input", prompt=prompt, answer=answer)
    return answer


def run(cmd, show=None):
    t0 = time.time()
    r = subprocess.run(cmd, capture_output=True, text=True)
    trace("exec", cmd=show or cmd, rc=r.returncode, secs=round(time.time() - t0, 2), stdout=r.stdout, stderr=r.stderr)
    return r


def snapshot():
    trace("state", rev=rev(), fields=fields, takes=takes, live=sorted(live()), proposal=proposal, thread_len=len(thread))


def ev(kind, **data):
    e = {"n": len(thread) + 1, "type": kind, **data}
    thread.append(e)
    THREAD.write(json.dumps(e, ensure_ascii=False) + "\n")
    trace("event", event=e)


def live():
    deleted = {e["take"] for e in thread if e["type"] == "TakeDeleted"}
    return {t: x for t, x in takes.items() if t not in deleted}


def definitions():
    text = (ROOT / "idea.md").read_text()
    m = re.search(r"\| Field \| What it says \|.*?\n\n", text, re.S)
    return m.group(0) if m else text


def transcribe(wav):
    r = run(["voxtype", "transcribe", str(wav)])
    lines = [l for l in r.stdout.splitlines() if l.strip()]
    return lines[-1].strip() if lines and not lines[-1].startswith("Processing") else ""


def record(path=None):
    tid = f"t{len(takes) + 1}"
    if path:
        wav = Path(path)
        if not wav.is_file():
            trace("no_such_file", path=str(wav))
            print(f"  no such file: {wav}")
            return
    else:
        wav = WORK / f"{tid}.wav"
        p = subprocess.Popen(["pw-record", "--rate", "16000", "--channels", "1", "--format", "s16", str(wav)])
        trace("record_start", wav=str(wav), pid=p.pid)
        ask("  recording... Enter to stop ")
        p.send_signal(signal.SIGINT)
        rc = p.wait()
        trace("record_stop", wav=str(wav), rc=rc, bytes=wav.stat().st_size if wav.exists() else None)
    t0 = time.time()
    print("  transcribing...")
    heard = transcribe(wav)
    secs = round(time.time() - t0, 1)
    text = review(heard)
    if text is None:
        trace("take_discarded", take=tid, wav=str(wav), heard=heard)
        print("  discarded; nothing entered the thread")
        return
    takes[tid] = text
    ev("TakeRecorded", take=tid, wav=str(wav), secs=secs, text=text, heard=heard, edited=text != heard)
    print(f"  {tid}: {text!r}")


def drain_inbox():
    if not INBOX.exists():
        return
    taken = WORK / f"inbox-{time.time_ns()}.jsonl"
    INBOX.rename(taken)
    entries = [json.loads(l) for l in taken.read_text().splitlines() if l.strip()]
    trace("inbox_taken", path=str(taken), count=len(entries))
    for entry in entries:
        tid = f"t{len(takes) + 1}"
        takes[tid] = entry["text"]
        audio = entry.get("audio") or {}
        wavs = [x["wav"] for x in audio.get("segments", []) if "wav" in x]
        ev("TakeRecorded", take=tid, source=entry["source"], session=entry.get("session"), at=entry["at"],
           text=entry["text"], wavs=wavs, audio_ok=audio.get("ok"), audio=audio)
        print(f"  {tid} from {entry['source']}: {entry['text'][:100]!r}")


def review(heard):
    text = heard
    while True:
        print(f"\n  heard: {text}\n  [Enter] approve  [e]dit  [x] discard")
        k = ask("  > ").strip()
        if k == "":
            return text
        if k == "x":
            return None
        if k == "e":
            readline.set_startup_hook(lambda: readline.insert_text(text))
            try:
                text = ask("  edit: ").strip()
            finally:
                readline.set_startup_hook()


PROMPT = """You place a person's dictated words into a five-field task prompt. Use only their words.

Field definitions:
{defs}

Current field texts (may be empty):
{current}

Transcripts of the person's takes, by id:
{transcripts}

Rules:
- Each field's text is built from what the person said. Every sentence in it carries a claim:
  {{"said": "<take id>", "span": "<exact words copied from that transcript>"}} or
  {{"added": "<why you added words they did not say>"}}. Prefer "said"; keep "added" minimal.
- If the person said nothing that fits a field, leave its text empty and ask one question for it.
- If what they said for a field does not meet the definition (a symptom, a guess, a pattern
  without a force), keep their words and ask a question that would make it meet it.

Reply with JSON only, no prose, no code fence:
{{"fields": {{"<field>": {{"text": "...", "claims": [ ... ]}}}}, "asked": [{{"field": "...", "question": "..."}}]}}
"""


def place():
    global proposal
    lt = live()
    if not lt:
        print("  no live takes")
        return
    prompt = PROMPT.format(
        defs=definitions(),
        current="\n".join(f"- {f}: {fields[f] or '(empty)'}" for f in FIELDS),
        transcripts="\n".join(f"[{t}] {x}" for t, x in lt.items()),
    )
    ev("JobSubmitted", base=rev(), ctx=sorted(lt))
    print(f"  asking {MODEL}...")
    trace("prompt", text=prompt)
    t0 = time.time()
    r = run(["claude", "-p", prompt, "--model", MODEL], show=["claude", "-p", "<prompt>", "--model", MODEL])
    out = r.stdout.strip()
    m = re.search(r"\{.*\}", out, re.S)
    try:
        proposal = json.loads(m.group(0))
    except Exception as e:
        trace("bad_reply", error=repr(e))
        print("  bad reply:\n" + out + r.stderr)
        return
    proposal["base"], proposal["ctx"] = rev(), sorted(lt)
    ev("ProposalMade", secs=round(time.time() - t0, 1), base=rev(), ctx=sorted(lt), fields=proposal["fields"])
    for q in proposal.get("asked", []):
        ev("Asked", field=q["field"], question=q["question"])


def rev():
    return sum(1 for e in thread if e["type"] in ("Typed", "Accepted"))


def check_span(c):
    t = c.get("said")
    if t is None:
        return "added"
    src = takes.get(t, "")
    return "ok" if c.get("span", "").lower().strip(" .") in src.lower() else "NOT IN TAKE"


def show():
    print("\n" + "=" * 78)
    print(f"rev {rev()}   takes live: {', '.join(live()) or '-'}")
    for f in FIELDS:
        print(f"\n## {f}\n  {fields[f] or '(empty)'}")
    if proposal:
        stale = proposal["base"] != rev() or not set(proposal["ctx"]) <= set(live())
        print("\n--- proposal" + ("  [STALE: accept refused]" if stale else "") + " ---")
        for f in FIELDS:
            p = proposal["fields"].get(f, {})
            print(f"\n## {f}\n  {p.get('text') or '(empty)'}")
            for c in p.get("claims", []):
                tag = f"said {c['said']}" if "said" in c else "added"
                print(f"    - {tag}: {c.get('span') or c.get('added')}   [{check_span(c)}]")
        for q in proposal.get("asked", []):
            print(f"\n  ? {q['field']}: {q['question']}")
    print("\n--- thread ---")
    for e in thread[-12:]:
        print("  " + json.dumps(e, ensure_ascii=False))
    print("\n[r]ecord  [f]ile wav  [p]lace  [a]ccept  [x]reject  [t]ype  [l]isten  [d]elete take  [s]how takes  [q]uit")


def accept():
    global proposal
    if not proposal:
        return
    if proposal["base"] != rev() or not set(proposal["ctx"]) <= set(live()):
        trace("accept_refused", base=proposal["base"], rev=rev(), ctx=proposal["ctx"], live=sorted(live()))
        print("  stale: revision moved or a take it used was deleted; place again")
        return
    for f in FIELDS:
        fields[f] = proposal["fields"].get(f, {}).get("text", "")
    ev("Accepted")
    proposal = None


def main():
    global proposal
    if LATEST.is_symlink() or not LATEST.exists():
        LATEST.unlink(missing_ok=True)
        LATEST.symlink_to(WORK)
    trace("start", work=str(WORK), model=MODEL, pid=os.getpid())
    print(f"PROTOTYPE voice-brief. Log in {WORK} (also {LATEST})")
    while True:
        drain_inbox()
        snapshot()
        show()
        try:
            k = ask("> ").strip()
        except EOFError:
            k = "q"
        if k == "q":
            trace("quit")
            print(f"trace and thread in {WORK}")
            return
        elif k == "r":
            record()
        elif k == "f":
            record(ask("  wav path: ").strip())
        elif k == "p":
            place()
        elif k == "a":
            accept()
        elif k == "x" and proposal:
            ev("Rejected", reason=ask("  reason: "))
            proposal = None
        elif k == "t":
            i = int(ask("  field 1-5: ")) - 1
            fields[FIELDS[i]] = ask(f"  {FIELDS[i]}: ")
            ev("Typed", field=FIELDS[i])
        elif k == "d":
            t = ask("  take id: ").strip()
            if t in live():
                ev("TakeDeleted", take=t, reason=ask("  reason: "))
        elif k == "l":
            t = ask("  take id: ").strip()
            e = next((e for e in thread if e["type"] == "TakeRecorded" and e["take"] == t), None)
            for w in (e or {}).get("wavs") or ([e["wav"]] if e and "wav" in e else []):
                run(["pw-play", w])
        elif k == "s":
            for t, x in takes.items():
                print(f"  {t}{' (deleted)' if t not in live() else ''}: {x}")
            ask("  Enter ")


if __name__ == "__main__":
    try:
        main()
    except BaseException as e:
        trace("crash", error=repr(e), tb=traceback.format_exc())
        raise
