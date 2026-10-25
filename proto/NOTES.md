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

- Claude Code records dictated prompts as typed in the hook input, `history.jsonl` and the session
  log.
- The first detector read Claude Code's debug log (`DEBUG=1`, line
  `[voice] Injecting transcript`). It was replaced on 2026-10-25 by the proxy (next section).
- `proto/vb_hook.py` runs on every prompt submitted in this project (from
  `.claude/settings.local.json`, gitignored). A prompt counts as dictated when it contains the
  final transcript of a voice stream that ended after this session's previous prompt and that no
  earlier prompt used. Case and whitespace don't matter.
  - `edited` means the prompt holds more than the dictated text.
  - A prompt that matches nothing is typed. That covers prompts Claude Code injects, such as
    task notifications, which the debug-log detector had counted as dictated.
- Dictated prompts go to `~/tmp/vb-proto-inbox.jsonl` and still reach Claude. Each decision is
  logged to `~/tmp/vb-proto-hook.jsonl`.
- The prototype drains the inbox before each screen, recording a `TakeRecorded` with
  `source: claude-code-voice`.
- Replayed on the five streams captured live on 2026-10-25:
  - two dictations in one prompt matched, unedited;
  - a dictation with a typed fix ("wgar") matched as edited;
  - the same stream was not used twice;
  - a typed prompt and a task notification stayed typed.

## Voice audio from Claude Code

Claude Code keeps no audio, and its stream-URL override (`VOICE_STREAM_BASE_URL`) is compiled
out. `proto/vb-claude.sh` starts `mitmdump` on 127.0.0.1:8899 if it isn't already running, then
runs `claude` with `HTTPS_PROXY` and the mitmproxy CA.

- Only `api.anthropic.com` is decrypted.
- `proto/vb_mitm.py` saves only the `voice_stream` WebSocket. Each stream gets a WAV of the
  client's binary frames (16 kHz mono linear16), a JSON file with the server's transcript
  messages, and a line in `~/tmp/vb-proto-audio/index.jsonl`. Nothing else is written.
- Each stream's index line holds its final transcript: the last `TranscriptText` of each
  utterance, joined. It also says whether the client sent `CloseStream` and the server sent
  `TranscriptEndpoint`; `ok` means both happened.
- Until the debug log was dropped, the hook also compared captured bytes with the debug log's
  chunk sizes. They were equal for every stream it checked.
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

## Prototype 2: the Claude Code loop

`vb_brief.py` answers a second question: does the loop of dictating in a brief pane, seeing the
idea placed, answering by voice, accepting and handing off feel right?

- `proto/vb-claude.sh --brief` starts a Claude Code session whose dictated prompts go into the
  idea only (the hook blocks them from the agent). Typed prompts still reach the agent.
- `python3 proto/vb_brief.py` in another pane picks up each dictation within a second.
  - `p` places every live take through `spike/isolation/run.sh isolated` and the Rust validator.
  - Each claim shows `said tN` or `ADDED: reason`; each empty field shows its question.
  - Answering is dictating again, then `p`.
  - `a` accepts. `h` writes `prompt.md` and copies it to the clipboard.
  - `e` corrects a take's text, `d` deletes a take, `l` plays its audio.
- Checked with piped input on 2026-10-25: two inbox takes, place (7.6 s), accept, then hand-off to
  the clipboard. Not yet run by the operator.
