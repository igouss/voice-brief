#!/usr/bin/env bash
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
port=8899
ca="$HOME/.mitmproxy/mitmproxy-ca-cert.pem"
if ! ss -ltn "sport = :$port" | grep -q LISTEN; then
  mitmdump --listen-host 127.0.0.1 -p "$port" --allow-hosts '^api\.anthropic\.com(:443)?$' \
    --set flow_detail=0 -s "$here/vb_mitm.py" >>"$HOME/tmp/vb-proto-mitm.out" 2>&1 &
  for _ in $(seq 50); do ss -ltn "sport = :$port" | grep -q LISTEN && break; sleep 0.1; done
fi
DEBUG=1 HTTPS_PROXY="http://127.0.0.1:$port" NODE_EXTRA_CA_CERTS="$ca" exec claude "$@"
