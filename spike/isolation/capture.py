import json, os, time
from pathlib import Path

OUT = Path(os.environ["SPIKE_CAPTURE_DIR"])


def request(flow):
    if not flow.request.path.startswith("/v1/messages"):
        return
    OUT.mkdir(parents=True, exist_ok=True)
    body = json.loads(flow.request.get_text() or "{}")
    (OUT / f"{time.time_ns()}.json").write_text(json.dumps({"path": flow.request.path, "body": body}, indent=1))
