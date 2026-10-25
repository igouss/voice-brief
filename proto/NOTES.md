# PROTOTYPE — delete when answered

`vb_proto.py` is throwaway. Run: `python3 proto/vb_proto.py`.

## Question

Do my dictated words land in the five fields, quoted, with a question where I said nothing? And
does the loop of saying something, seeing it placed, and correcting it feel right?

## What it is

A stdlib Python REPL. It uses `pw-record` (16 kHz mono) and `voxtype transcribe` (parakeet,
loaded on each call). Then `claude -p` places the words into the five fields. The field
definitions are read from `idea.md` at runtime. The thread is kept in memory and written to the
temp directory when you quit.

Not in it: ratatui, `vbd`, celld, Herdr, git, model residency, or the re-check of claims that cite
a deleted take.

## Seen on the first run (synthetic speech, 2026-10-25)

- Every field I filled came back as `said t1` with exact spans. "Would reverse it" came back empty
  with a question.
- The question offered example answers ("for example, the desktop build still pulls in the HTTP
  client"). That is the AI suggesting content in the field it must not guess.
- voxtype heard "reqwest" as "request", and the AI carried it into Force. So correcting a
  transcript matters from the first take.
- Deleting the take after accepting left the fields built from it unchanged. The design's
  re-check is needed.
- Timing: transcription about 3.7 s, of which 2.4 s is loading the model; placement about 14 s.

## Seen on the first live run (Iouri's mic, 2026-10-25)

Takes: t1 a mic test, t2 "Yeah.", t3 an empty take (a mistyped `f` path recorded silently, now
refused), t4 a real idea about 50 s long.

- **The take names the fields as markers** ("Weakest part … Force … smallest move … preserves …
  would reverse it"). It's a natural way to dictate, and the placement can use it.
- **Speech-to-text loses the jargon.** "clawed" is probably `~/.claude`; "overwriting boosters
  work" is probably "overwriting loses work"; "would reserve it" is "would reverse it"; "DevBox is
  trapped" is unclear. Parakeet has no vocabulary for paths and project names.
- **The AI didn't guess "clawed"; it asked which file.** That keeps fidelity but costs a round,
  where correcting the transcript would have fixed it in one.
- **A suggestion inside the question happened twice out of two runs.** The Force question offered
  "for example, Fedora's copy has to stay immutable, but it also has to match the DevBox copy";
  the reversal question suggested its own answer.
- On the mic-test takes, it asked all five questions and filled nothing. Correct.
- Placement took 7 to 13 s; transcribing 50 s of audio took 5.5 s, model load included.
- The thread didn't record the proposal's field texts, so a run can't be reviewed afterwards.
  Fixed: `ProposalMade` and `TakeRecorded` now carry them.

## Verdict

- Transcript review comes first. Iouri, 2026-10-25: "Yeah, transcription is weak, I would need to
  view and approve it before it enters into the system." The prototype now shows each transcript
  and offers approve, edit or discard. Only an approved text is appended as `TakeRecorded`, which
  keeps both what voxtype heard and the approved text.
- (placement fidelity: still to judge on a live take)

## Claude Code hook

Only prompts dictated with Claude Code's `/voice` go into the idea; typed prompts don't.

- Claude Code doesn't mark dictated prompts. The hook input, `history.jsonl` and the session log
  all record them as typed (`promptSource: "typed"`).
- The only local signal is Claude Code's debug log. When a dictation is put into the input box it
  writes `[voice] Injecting transcript (N chars)`.
- No `settings.json` key turns that log on; an `env` block in settings is read too late. It has
  to be set in the environment before Claude Code starts: `DEBUG=1` writes
  `~/.claude/debug/<session_id>.txt`.
- `proto/vb_hook.py` runs on every prompt submitted in this project (from
  `.claude/settings.local.json`, gitignored). It checks whether that session's debug log gained a
  `[voice] Injecting transcript` line since the last submit. If so, it copies the prompt to
  `~/tmp/vb-proto-inbox.jsonl`; the prompt still goes to Claude.
- Each decision is logged to `~/tmp/vb-proto-hook.jsonl`.
- The prototype drains the inbox before each screen, recording a `TakeRecorded` with
  `source: claude-code-voice`.
- Seen live on 2026-10-25: a typed prompt was classified "typed"; a dictated one was "dictated", and the logged character count (95) matched the prompt. A dictation that you then edit by keyboard still counts as dictated.

## Voice audio from Claude Code

Claude Code keeps no audio, and its stream-URL override (`VOICE_STREAM_BASE_URL`) is compiled
out. `proto/vb-claude.sh` starts `mitmdump` on 127.0.0.1:8899 if it isn't already running, then
runs `claude` with `DEBUG=1`, `HTTPS_PROXY` and the mitmproxy CA.

- Only `api.anthropic.com` is decrypted.
- `proto/vb_mitm.py` saves only the `voice_stream` WebSocket. Each stream gets a WAV of the
  client's binary frames (16 kHz mono linear16), a JSON file with the server's transcript
  messages, and a line in `~/tmp/vb-proto-audio/index.jsonl`. Nothing else is written.
- The hook matches each recording span in the debug log to a captured stream. It counts a
  recording as exact only when the captured bytes equal the sum of the "Sending audio chunk"
  sizes in the log.
- On 12 dictations, the bytes sent were 0.92 to 0.99 of the span between "Starting recording
  session" and "Recording stopped".
- Checked:
  - `claude -p` works through the proxy;
  - the hook handled the exact, mismatch, missing-capture and typed cases on synthetic logs.
- Seen live on 2026-10-25: one prompt made of two dictations went through the proxy. Both streams
  were captured exactly: 241,604 bytes in 172 chunks, and 509,146 bytes in 368 chunks, equal to
  the debug log's sums. Running voxtype on the saved WAVs gave the spoken words back.
- While `vb-claude.sh` is in use, the proxy decrypts all of that session's traffic with
  Anthropic, including the OAuth token. It writes none of it to disk.
