#!/usr/bin/env bash
set -euo pipefail
repo="$(git -C "$(dirname "$0")" rev-parse --show-toplevel)"
ca="$HOME/.mitmproxy/mitmproxy-ca-cert.pem"
unit="$HOME/.config/systemd/user/vb-voice-proxy.service"
local_settings="$repo/.claude/settings.local.json"

command -v mitmdump >/dev/null || { echo "mitmdump not found: uv tool install mitmproxy" >&2; exit 1; }

if [ ! -f "$ca" ]; then
  mitmdump --listen-host 127.0.0.1 -p 0 >/dev/null 2>&1 &
  for _ in $(seq 50); do [ -f "$ca" ] && break; sleep 0.1; done
  kill $! 2>/dev/null || true
fi
[ -f "$ca" ] || { echo "mitmproxy did not create $ca" >&2; exit 1; }

mkdir -p "$(dirname "$unit")"
[ -L "$unit" ] && rm "$unit"
sed "s|@REPO@|$repo|" "$repo/proto/vb-voice-proxy.service.in" >"$unit"
systemctl --user daemon-reload
systemctl --user enable vb-voice-proxy.service
systemctl --user restart vb-voice-proxy.service

[ -f "$local_settings" ] || echo '{}' >"$local_settings"
jq --arg ca "$ca" --arg repo "$repo" \
  '.env.NODE_EXTRA_CA_CERTS = $ca
   | .extraKnownMarketplaces["voice-brief"] = {source: {source: "directory", path: $repo}}' \
  "$local_settings" >"$local_settings.tmp"
mv "$local_settings.tmp" "$local_settings"

cargo build -q --bins --manifest-path "$repo/spike/contract/Cargo.toml"

echo "installed: $unit, $local_settings; proxy $(systemctl --user is-active vb-voice-proxy.service) on 127.0.0.1:8897"
