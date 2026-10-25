import glob, json, sys
from pathlib import Path

LEAKS = ["False green", "Diagnose before you design", "MEMORY.md", "voice-brief", "skills"]


def bodies(run):
    return [json.load(open(f))["body"] for f in sorted(glob.glob(f"{run}/requests/*.json"))]


def leaks(run):
    found = {}
    for b in bodies(run):
        text = json.dumps(b)
        for needle in LEAKS:
            if needle in text:
                found[needle] = found.get(needle, 0) + text.count(needle)
    return found


def main():
    expect, run = sys.argv[1], sys.argv[2]
    if not bodies(run):
        print(f"FAIL {run}: no captured request")
        sys.exit(1)
    found = leaks(run)
    ok = bool(found) if expect == "leaks" else not found
    print(f"{'ok  ' if ok else 'FAIL'} {Path(run).name}: expected {expect}, found {found or 'none'}")
    sys.exit(0 if ok else 1)


main()
