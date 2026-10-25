import json, os, subprocess, time, wave
from pathlib import Path

from mitmproxy import http

VOICE_PATH = "/api/ws/speech_to_text/voice_stream"
DATA = Path(os.environ.get("XDG_DATA_HOME", Path.home() / ".local" / "share")) / "voice-brief" / "projects"
RATE = 16000

streams = {}


def port_of(endpoint):
    return int(endpoint.rsplit(":", 1)[1], 16)


def client_inodes(tables, client_port, proxy_port):
    inodes = set()
    for line in tables.splitlines():
        cols = line.split()
        if len(cols) > 9 and cols[0].endswith(":") and port_of(cols[1]) == client_port and port_of(cols[2]) == proxy_port:
            inodes.add(cols[9])
    return inodes


def client_pid(client_port, proxy_port):
    inodes = client_inodes("".join(open(t).read() for t in ("/proc/net/tcp", "/proc/net/tcp6")), client_port, proxy_port)
    for pid in filter(str.isdigit, os.listdir("/proc")):
        try:
            for fd in os.listdir(f"/proc/{pid}/fd"):
                if os.readlink(f"/proc/{pid}/fd/{fd}").removeprefix("socket:[").rstrip("]") in inodes:
                    return int(pid)
        except OSError:
            continue
    return None


def project_of(pid):
    try:
        cwd = os.readlink(f"/proc/{pid}/cwd")
    except (OSError, TypeError):
        return None
    r = subprocess.run(["git", "-C", cwd, "rev-parse", "--show-toplevel"], capture_output=True, text=True)
    return r.stdout.strip() if r.returncode == 0 else cwd


def slug(path):
    return path.replace("/", "-") if path else "_unknown"


def final_transcript(server_text):
    utterances, current = [], ""
    for m in server_text:
        msg = json.loads(m["text"]) if m["text"].startswith("{") else {}
        if msg.get("type") == "TranscriptText":
            current = msg.get("data", "")
        elif msg.get("type") == "TranscriptEndpoint" and current:
            utterances.append(current)
            current = ""
    if current:
        utterances.append(current)
    return " ".join(utterances)


def is_voice(flow):
    return flow.request.path.startswith(VOICE_PATH)


def websocket_start(flow: http.HTTPFlow):
    if not is_voice(flow):
        return
    pid = client_pid(flow.client_conn.peername[1], flow.client_conn.sockname[1])
    project = project_of(pid)
    base = DATA / slug(project)
    (base / "open").mkdir(parents=True, exist_ok=True)
    name = time.strftime("%Y%m%dT%H%M%S") + f"-{flow.id[:8]}"
    (base / "open" / name).touch()
    streams[flow.id] = {"name": name, "start": time.time(), "audio": bytearray(), "chunks": 0, "pid": pid,
                        "project": project, "base": base,
                        "client_text": [], "server_text": []}


def websocket_message(flow: http.HTTPFlow):
    s = streams.get(flow.id)
    if s is None:
        return
    m = flow.websocket.messages[-1]
    if m.from_client and not m.is_text:
        s["audio"] += m.content
        s["chunks"] += 1
    elif m.from_client:
        s["client_text"].append({"t": time.time(), "text": m.text})
    else:
        s["server_text"].append({"t": time.time(), "text": m.text if m.is_text else f"<{len(m.content)} bytes>"})


def websocket_end(flow: http.HTTPFlow):
    s = streams.pop(flow.id, None)
    if s is None:
        return
    audio = s["base"] / "audio"
    audio.mkdir(parents=True, exist_ok=True)
    wav = audio / f"{s['name']}.wav"
    with wave.open(str(wav), "wb") as w:
        w.setnchannels(1)
        w.setsampwidth(2)
        w.setframerate(RATE)
        w.writeframes(bytes(s["audio"]))
    meta = {"name": s["name"], "start": s["start"], "end": time.time(), "wav": str(wav), "bytes": len(s["audio"]),
            "chunks": s["chunks"], "pid": s["pid"], "project": s["project"],
            "final": final_transcript(s["server_text"]),
            "closed_by_client": any('"CloseStream"' in m["text"] for m in s["client_text"]),
            "endpoint": any('"TranscriptEndpoint"' in m["text"] for m in s["server_text"]),
            "query": flow.request.query.fields and dict(flow.request.query),
            "client_text": s["client_text"], "server_text": s["server_text"]}
    (s["base"] / "open" / s["name"]).unlink(missing_ok=True)
    (audio / f"{s['name']}.json").write_text(json.dumps(meta, ensure_ascii=False, indent=1))
    with (s["base"] / "index.jsonl").open("a") as f:
        f.write(json.dumps({k: meta[k] for k in ("name", "start", "end", "wav", "bytes", "chunks", "pid", "project",
                                                   "final", "closed_by_client", "endpoint")},
                           ensure_ascii=False) + "\n")
