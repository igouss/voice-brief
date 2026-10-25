#!/usr/bin/env bash
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
port=8897
ca="$HOME/.mitmproxy/mitmproxy-ca-cert.pem"
extra=()
if [ "${1:-}" = "--brief" ]; then
  export VB_BRIEF=1
  extra=(--plugin-dir "$here/voice-notes")
  shift
fi
systemctl --user start vb-voice-proxy.service
for _ in $(seq 50); do ss -ltn "sport = :$port" | grep -q LISTEN && break; sleep 0.1; done
if ! ss -ltn "sport = :$port" | grep -q LISTEN; then
  echo "vb-voice-proxy is not listening on 127.0.0.1:$port; see: journalctl --user -u vb-voice-proxy" >&2
  exit 1
fi
HTTPS_PROXY="http://127.0.0.1:$port" NODE_EXTRA_CA_CERTS="$ca" exec claude "${extra[@]}" "$@"
