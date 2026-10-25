#!/usr/bin/env bash
set -euo pipefail
mode="$1"; system_prompt="$(cat "$2")"; prompt="$(cat "$3")"; mkdir -p "$4"; out="$(cd "$4" && pwd)"
here="$(cd "$(dirname "$0")" && pwd)"
port=8898
mkdir -p "$out/requests"
SPIKE_CAPTURE_DIR="$out/requests" mitmdump --listen-host 127.0.0.1 -p "$port" \
  --allow-hosts '^api\.anthropic\.com(:443)?$' --set flow_detail=0 -s "$here/capture.py" >"$out/proxy.log" 2>&1 &
proxy=$!
trap 'kill $proxy' EXIT
for _ in $(seq 50); do ss -ltn "sport = :$port" | grep -q LISTEN && break; sleep 0.1; done
model="${MODEL:-claude-opus-5-5}"
effort="${EFFORT:-medium}"
printf '{"mode": "%s", "model": "%s", "effort": "%s", "claude": "%s", "system_sha256": "%s", "prompt_sha256": "%s", "started": "%s"}\n' \
  "$mode" "$model" "$effort" "$(claude --version | head -1)" \
  "$(printf '%s' "$system_prompt" | sha256sum | cut -c1-64)" "$(printf '%s' "$prompt" | sha256sum | cut -c1-64)" \
  "$(date -u +%Y-%m-%dT%H:%M:%SZ)" >"$out/call.json"
proxy_env=(HTTPS_PROXY="http://127.0.0.1:$port" NODE_EXTRA_CA_CERTS="$HOME/.mitmproxy/mitmproxy-ca-cert.pem")
case "$mode" in
  default)
    env "${proxy_env[@]}" claude -p "$prompt" --model "$model" --effort "$effort" >"$out/reply.txt"
    ;;
  isolated)
    empty="$(mktemp -d)"
    cd "$empty"
    env "${proxy_env[@]}" CLAUDE_CODE_DISABLE_CLAUDE_MDS=1 CLAUDE_CODE_DISABLE_AUTO_MEMORY=1 \
      claude -p "$prompt" --model "$model" --effort "$effort" \
      --system-prompt "$system_prompt" --setting-sources "" --tools "" \
      --strict-mcp-config --disable-slash-commands --no-session-persistence --exclude-dynamic-system-prompt-sections >"$out/reply.txt"
    rmdir "$empty"
    ;;
  *) echo "mode: default | isolated" >&2; exit 2 ;;
esac
