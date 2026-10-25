# Contract as a Worker (workers-rs)

`vb-contract` wrapped for celld with the standard workers-rs toolchain. The `worker` crate and
`worker-build` 0.8.7 produce `build/index_bg.wasm` (308 KB) and its JS glue. The wrapper is its own
crate so the core never depends on `worker` or `wasm-bindgen`.

`POST /` with `{"take", "transcript", "reply"}` returns 200 and the events, or 422 and the
violations.

Seen under workerd (`npx wrangler@4.149.0 dev`), 2026-10-25: the smoke reply returned 200 with
`ProposalMade` and four `Asked` events. The same reply with one span edited returned 422
`span_not_in_transcript`. celld itself is not installed here, so this has not run on celld.
