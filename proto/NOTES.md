# Prototypes

Throwaway code that answers questions. What is current:

| Piece | Run | What it is |
|---|---|---|
| `vb-voice-proxy.service` | `systemctl --user status vb-voice-proxy` | mitmdump on 127.0.0.1:8897 with `vb_mitm.py`, a systemd user service (symlinked into `~/.config/systemd/user/`) |
| `vb-claude.sh` | `proto/vb-claude.sh --brief` | starts the service if needed, then `claude` through it; `--brief` makes the notes pane |
| `voice-notes/` | loaded by `--brief` | the Claude Code mod: notes, placement, hand-off |

Captures and notes live per Git project under `~/.local/share/voice-brief/projects/` (`design.md`,
"Where the data lives").

Retired, in git history: `vb_proto.py` (prototype 1, local voxtype), `vb_hook.py` (prompt hook),
`vb_brief.py` (prototype 2, terminal app). The sections below keep what each showed.

## Prototype 1: local capture (retired)

Question: do my dictated words land in the five fields, quoted, with a question where I said
nothing, and does the say/see/correct loop feel right?

A stdlib Python REPL: `pw-record`, `voxtype transcribe`, then `claude -p` with the definitions
read from `idea.md`.

- Every field I filled came back as `said` with exact spans; an unsaid field came back empty with
  a question.
- Questions offered example answers in both runs ("for example, the desktop build still pulls in
  the HTTP client"). The contract prompt in `spike/contract/prompt/system.md` now forbids that.
- **Local speech-to-text loses the jargon:** "reqwest" became "request", `~/.claude` became
  "clawed", "would reverse it" became "would reserve it".
- Deleting a take after accepting left the fields built from it unchanged: the re-check in the
  design is needed.
- Speaking the field names as markers is a natural way to dictate.
- Timing: 50 s of audio took 5.5 s to transcribe, model load included; placement took 7 to 14 s.
- **Verdict:** transcript review comes first. Iouri, 2026-10-25: "Yeah, transcription is weak, I
  would need to view and approve it before it enters into the system."

## Telling dictated prompts from typed ones

Claude Code records dictated prompts as typed in the hook input, `history.jsonl` and the session
log.

- First detector: Claude Code's debug log (`DEBUG=1`, line `[voice] Injecting transcript`).
- Second: a prompt is dictated when it contains the final transcript of an unused voice stream
  from the proxy. Replayed on five live streams:
  - two dictations in one prompt matched;
  - a dictation with a typed fix matched as edited;
  - a typed prompt and a task notification stayed typed.
- Now: capture doesn't depend on the prompt at all (prototype 3). The mod marks a note sent when
  a submitted prompt contains its text.

## Voice audio from Claude Code

Claude Code keeps no audio, and its stream-URL override (`VOICE_STREAM_BASE_URL`) is compiled out.
The proxy decrypts only `api.anthropic.com` and saves only the `voice_stream` WebSocket.

- Each stream gets:
  - a WAV of the client's binary frames (16 kHz mono linear16);
  - a JSON file with every server message;
  - a row in its project's `index.jsonl`: final transcript, client pid, project, notes pane or
    not, clean close on both sides.
- The final transcript is the last `TranscriptText` of each utterance, joined.
- The client pid comes from the connection's socket inode in `/proc`. The project is the Git root
  of that process's working directory, or the directory itself outside a repo.
- Seen live, 2026-10-25:
  - **Captured bytes equal the sent bytes.** While the debug log was on, two streams' captured
    bytes equalled the logged chunk sums exactly: 241,604 bytes in 172 chunks, and 509,146 bytes
    in 368 chunks.
  - voxtype transcribed the saved WAVs back to the spoken words.
  - Across 12 dictations, the bytes sent were 0.92 to 0.99 of the time between "Starting
    recording session" and "Recording stopped".
- While a session runs through the proxy, the proxy decrypts all of that session's traffic with
  Anthropic, including the OAuth token. It writes none of it to disk.
- The proxy first ran inside the pane that started it, so closing that pane cut every proxied
  session off. Since 2026-10-25 it is `vb-voice-proxy.service`. *Checked:* `claude -p` works
  through it, and its argv carries the host pattern intact.

## Prototype 2: the Claude Code loop in a terminal app (retired)

`vb_brief.py` read an inbox the proxy and the hook wrote, and placed, accepted and handed off from
a terminal screen.

- The notes pane (`--brief`, `VB_BRIEF=1`) captured dictations as soon as the stream ended; Enter
  was optional.
- Live, 2026-10-25: one note sent, one not sent, both captured.
- Its placement of four chatter takes left all five fields empty, each with a question that
  suggested no answer.

Operator, 2026-10-25: "I don't want to send the dictated text to the model when I don't want to,
and if I do want to send the dictated text, I will just press enter … For now, a preferred mode
where I just open Claude and use its voice … for this application to populate voice notes."

## Prototype 3: everything inside Claude Code (current)

Operator, 2026-10-25: "Maybe we could just use Claude mods and this void capture and processing,
and we just do it from the inside Claude code. Maybe we don't need to write an application right
now."

`proto/voice-notes/` is a Claude Code mod. `vb-claude.sh --brief` loads it with `--plugin-dir`,
and it acts only where `VB_BRIEF=1`.

- On start it finds its own `claude` process and its project's Git root. Every second it reads
  that project's `index.jsonl` for streams of its process since the session began.
- Each new stream becomes a note: a toast, a status line, and a "captured" band above the prompt.
- **The `Voice notes` pane:**
  - lists the notes; ↗ marks the ones sent to Claude, and × deletes one;
  - has Place, Accept, Reject and Hand off buttons;
  - shows the proposal or the accepted revision field by field: each claim with `said tN` or
    `added: reason`, and each empty field with its question.
- Commands: `/vb`, `/vb-place`, `/vb-accept`, `/vb-reject`, `/vb-handoff`, `/vb-delete tN`,
  `/vb-fix tN <text>`.
- **Placement** runs `spike/isolation/run.sh isolated` and the Rust validator, found beside the
  mod, so a notes pane works in any repository.
- **Hand-off** copies the prompt to the clipboard (`$.ui.copy`) and writes `prompt.md`.
- Every event goes to `notes/<session start>/thread.jsonl` in the project's data folder.
- Checked 2026-10-25:
  - `claude plugin validate` passes, and `tsc` against this build's API is clean;
  - `claude plugin test`: a stream from this process becomes `t1`, and one from another process
    stays out. The first test was seen red with the pid changed.
  - Headless `claude -p` with the mod:
    - from this repo, it recorded its own pid (the launched process) and the project, and
      registered the seven commands;
    - from `/tmp`, it filed under `-tmp`.
- Not yet run in the operator's notes pane.
