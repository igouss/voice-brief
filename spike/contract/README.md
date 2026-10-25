# Output contract and validator

Spike bead `voice-brief-drafting-spike-ong.3`. One task: a transcript becomes the five fields.
Each field is either claims, each `said` (verbatim spans) or `added` (a reason), or empty with a
question. Never both.

- `prompt/system.md`: the system prompt. `{{definitions}}` is replaced with the format table read
  from `idea.md` (`render`), so the definitions have one source.
- `src/lib.rs`: the wire shape the model must reply with (`WireOutput`), the types it parses into
  (`Placement` is `Claims` or `Asked`, so both or neither cannot be represented), `validate`, and
  `events`. `events` turns a valid reply into a `ProposalMade` of kind `place` plus one `Asked`
  per empty field, as in `design.md`.
- `src/bin/render.rs`, `src/bin/validate.rs`: the file-reading shell around the core.

## What the validator rejects

| Violation | When |
|---|---|
| `Malformed` | not one JSON object of the wire shape (a code fence counts) |
| `UnknownField`, `MissingField` | a key other than the five, or one of the five absent |
| `EmptyWithoutQuestion` | no claims and no question, or a blank question |
| `ClaimsAndQuestion` | both |
| `ClaimWithoutSource`, `ClaimWithBothSources` | neither or both of `said` and `added` |
| `DuplicateClaimId` | an id used twice anywhere in the reply |
| `EmptySpan` | a `said` span that is blank after normalisation |
| `SpanNotInTranscript` | a `said` span not found in the transcript |

Span matching is case-sensitive. Before comparing, both sides get curly quotes and apostrophes
replaced with straight ones, and every whitespace run collapsed to one space. Tests:
`accepts_a_straight_apostrophe_for_a_curly_one`, `accepts_a_span_whose_whitespace_differs` and
`rejects_a_span_that_differs_only_in_case`.

The validator cannot judge whether a question proposes an answer; that belongs to the checker
(`.4`).

## Checks

- `cargo test`: 23 tests covering zero, one and many claims and spans, and one test per
  rejection. Each test is straight-line.
- `cargo mutants --file src/lib.rs` (2026-10-25): 22 mutants, 17 caught, 5 unviable, 0 missed.
- `cargo clippy --all-targets` is clean, with `wildcard_enum_match_arm` denied.

## Model calls

`../isolation/run.sh isolated` makes every call. The model and effort are configuration
(`MODEL`, `EFFORT`; the defaults are `claude-opus-5-5` and `medium`). Each call writes
`call.json` with the model, effort, Claude Code version, the prompt hashes and the start time.

Smoke run `../runs/smoke-t3-20261025T091609`: the reply validated and produced a `ProposalMade`
plus four `Asked` events. `verify.py` found the request clean.
