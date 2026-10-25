#!/bin/sh
set -u
cd "$(dirname "$0")"
meta="${TMPDIR:-/tmp}/voice-brief-tlc.$$"
status=0

run() {
  config=$1
  expect=$2
  out=$(tlc -workers 4 -deadlock -config "$config.cfg" -metadir "$meta/$config" JobThread 2>&1)
  if [ "$expect" = pass ]; then
    if printf '%s\n' "$out" | grep -q "No error has been found"; then
      echo "ok    $config passes"
    else
      echo "FAIL  $config should pass"; printf '%s\n' "$out" | grep -E "Error|violated" | head -3; status=1
    fi
  else
    if printf '%s\n' "$out" | grep -q "$expect"; then
      echo "ok    $config fails: $expect"
    else
      echo "FAIL  $config should fail with: $expect"; printf '%s\n' "$out" | grep -E "Error|violated|No error" | head -3; status=1
    fi
  fi
}

run Safety pass
run Liveness pass
run RedNoDedupe "Invariant AtMostOneResult is violated"
run RedNoGuard "Invariant AcceptOnlyFresh is violated"
run RedRewriteOnDelete "Action property AppendOnly is violated"
run RedVolatileSubmit "Temporal properties were violated"
rm -rf "$meta"
exit $status
