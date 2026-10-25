import json, time, wave
from pathlib import Path

from mitmproxy import http

VOICE_PATH = "/api/ws/speech_to_text/voice_stream"
OUT = Path.home() / "tmp" / "vb-proto-audio"
INDEX = OUT / "index.jsonl"
RATE = 16000

streams = {}


def is_voice(flow):
    return flow.request.path.startswith(VOICE_PATH)


def websocket_start(flow: http.HTTPFlow):
    if not is_voice(flow):
        return
    OUT.mkdir(parents=True, exist_ok=True)
    name = time.strftime("%Y%m%dT%H%M%S") + f"-{flow.id[:8]}"
    streams[flow.id] = {"name": name, "start": time.time(), "audio": bytearray(), "chunks": 0,
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
    wav = OUT / f"{s['name']}.wav"
    with wave.open(str(wav), "wb") as w:
        w.setnchannels(1)
        w.setsampwidth(2)
        w.setframerate(RATE)
        w.writeframes(bytes(s["audio"]))
    meta = {"name": s["name"], "start": s["start"], "end": time.time(), "wav": str(wav), "bytes": len(s["audio"]),
            "chunks": s["chunks"], "query": flow.request.query.fields and dict(flow.request.query),
            "client_text": s["client_text"], "server_text": s["server_text"]}
    (OUT / f"{s['name']}.json").write_text(json.dumps(meta, ensure_ascii=False, indent=1))
    with INDEX.open("a") as f:
        f.write(json.dumps({k: meta[k] for k in ("name", "start", "end", "wav", "bytes", "chunks")}) + "\n")
