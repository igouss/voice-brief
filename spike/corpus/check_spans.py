import json, sys

FIELDS = {"Weakest part", "Force", "Smallest move", "Preserves", "Would reverse it", "none"}


def problems(corpus):
    out = []
    for item in corpus:
        for c in item["claims"]:
            if c["span"] not in item["transcript"]:
                out.append(f"{item['id']}: span not in transcript: {c['span']!r}")
            if c["field"] not in FIELDS:
                out.append(f"{item['id']}: unknown field {c['field']!r}")
    empties = [set(i["empty"]) for i in corpus]
    if not any("Would reverse it" in e for e in empties):
        out.append("no transcript leaves Would reverse it empty")
    if not any("Preserves" in e for e in empties):
        out.append("no transcript leaves Preserves empty")
    return out


found = problems(json.load(open(sys.argv[1])))
print("\n".join(found) or "ok: every span is verbatim; both required gaps present")
sys.exit(1 if found else 0)
