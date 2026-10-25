# voice-brief — design

Develop an idea by voice, with AI help, into a task prompt for an agent, and hand the prompt to
an agent in the project you are working on. The problem, the prompt format and its lineage are in
[`idea.md`](idea.md).

Status: prototype. Claims marked *checked* were run on this machine on 2026-10-25; everything
else is intended behaviour. The prototypes and what each showed are in
[`proto/NOTES.md`](proto/NOTES.md); the drafting spike is in [`spike/`](spike/).

## Diagnosis (cm-dr0)

The riskiest assumption is still that the model can place my dictated words into the five fields
while separating what I said from what it adds, and ask instead of inventing a field I have not
filled. The drafting spike measures it (`spike/`, bead `voice-brief-drafting-spike-ong`). The
contract and validator are built; the checker and the full run are next.

The design changed under it on 2026-10-25 (see [decisions.md](decisions.md)). Speech-to-text and the interface no
longer need building first: Claude Code's `/voice` transcribes, and a Claude Code mod is the
interface.

## Two modes

| Mode | Capture | Interface | Status |
|---|---|---|---|
| **Claude Code** (now) | Claude Code's `/voice`, its audio and transcript saved by a local proxy | the `voice-notes` Claude Code plugin | plugin 0.2.0 |
| **Local** (later) | voxtype on this CPU, `pw-record` for audio | to be decided; the desktop design below | not started |

## The unit: an idea is a task prompt

The five fields (Weakest part, Force, Smallest move, Preserves, Would reverse it) and what each
must say are defined once, in `idea.md`'s format table (cm-eds). Weakest part, force and smallest
move come from cm-dr0; preserves and would reverse it from cm-e2y.

## Claude Code mode

```
claude, at a project's root                                  vb-voice-proxy.service (systemd user)
┌───────────────────────────────────────────────┐ HTTPS_PROXY ┌───────────────────────────────────┐
│ /voice dictation ── voice_stream WebSocket ───┼───────────► │ mitmdump 127.0.0.1:8897           │──► Anthropic
│                                               │             │  decrypts api.anthropic.com only  │
│ voice-notes plugin (plugins/voice-notes)      │             │  vb_mitm.py: per voice stream,    │
│  reads its project's index.jsonl ◄────────────┼── $.fs ─────│  WAV + transcript, filed under    │
│  pane: notes, Place / Accept / Hand off       │             │  the Git root of the client's     │
│  Place: $.model.complete + contract.ts        │             │  claude process                   │
│  /vb-setup: installs the proxy for a project  │             └───────────────────────────────────┘
└───────────────────────────────────────────────┘
```

- **The plugin** is `plugins/voice-notes`, listed in this repo's marketplace
  (`.claude-plugin/marketplace.json`): `/plugin install voice-notes --marketplace
  igouss/voice-brief`. It is self-contained and uses Claude Code's own facilities only: `$.fs` for
  files and `/proc/self/stat`, `$.model.complete` for placement, `$.ui` for the pane, the band, the
  status line and the clipboard. The one spawned command is `systemctl --user` in `/vb-setup`.
- **`/vb-setup`**, run once per project at its root, copies the proxy addon to
  `~/.local/share/voice-brief/vb_mitm.py`, writes
  `~/.config/systemd/user/vb-voice-proxy.service` from `setup/vb-voice-proxy.service.in`, enables
  and restarts it, waits for the mitmproxy CA, and writes `HTTPS_PROXY` and `NODE_EXTRA_CA_CERTS`
  into the project's `.claude/settings.local.json`. A restart of Claude Code there starts
  capture. Settings `env` values are not expanded (`$HOME`, `${HOME}` and `~` all failed,
  *checked*), so the written paths are absolute. *Checked:* `/vb-setup` run headless here
  installed the unit and the settings, and the service came up.
- **The proxy** runs as a systemd user service, enabled at login and restarted on failure, so
  closing any pane leaves it running. It decrypts only `api.anthropic.com` and writes only voice
  streams. If it is down, every proxied session hangs on its first request.
- **A voice stream** is tied to the `claude` process that opened it: the proxy finds the process
  from the client socket in `/proc`, then its working directory's Git root. The plugin keeps the
  streams whose pid is its own, read from `/proc/self/stat` (*checked:* equal to the launched
  pid).
- **Capture needs no Enter.** A dictation becomes a note as soon as Anthropic finishes
  transcribing it. Enter sends it to the session's agent as usual, and the note is marked sent.
- **Tap mode never sends a dictation.** With `voice.mode` `tap` in the project's settings, Claude
  Code submits each transcript when recording stops. The plugin's `prompt.submit` hook drops a
  prompt that equals a dictation just captured from this process (waiting up to 3 s for the
  proxy's row) and clears the prompt box if it still holds that text. Force: tap mode is what an
  outside key press can drive (one press starts, one stops; a held key cannot be faked from
  outside), and "the message was sent to the agent. I don't want that." Would reverse it: a way
  to send a dictation on purpose in tap mode. Checked 2026-10-25: Space sent to a test session's
  window with `hyprctl` started and stopped recording; two takes fed through a virtual microphone
  became t1 and t2, both dropped, the box empty, the agent idle. Gate:
  `plugins/voice-notes/tests/capture.test.tsx` ("in tap mode …").
- **A stream belongs to the process connected to the proxy.** The addon finds the client by the
  TCP socket whose local port is the client's and whose remote port is the proxy's. Matching the
  local port alone filed a dictation under an unrelated process that had a socket on the same
  port number elsewhere (seen 2026-10-25). Gate: `plugins/voice-notes/setup/test_vb_mitm.py`.
  *Checked* live: one note sent, one left unsent.
- **The plugin is active wherever it is enabled.** A session whose `HTTPS_PROXY` is not the voice
  proxy shows "not capturing" and points to `/vb-setup`. In this repo `.claude/settings.json`
  enables it and sets `HTTPS_PROXY`; `.claude/settings.local.json` (gitignored) holds the CA path
  and the marketplace's absolute path. Claude Code reads project settings only from the
  directory it starts in, so start it at the repo root.
- **The final transcript** is the last `TranscriptText` of each utterance, joined. The endpoint
  and its message types (`TranscriptText`, `TranscriptEndpoint`, `CloseStream`) are undocumented;
  a Claude Code update can change them, and then no dictation becomes a note. Each index row
  records whether the stream closed cleanly; a verifier that alarms on unclean streams is still to
  build.
- **Placement** is one `$.model.complete` call (`claude-opus-5-5`, effort medium) with only the
  plugin's system prompt (`prompt/system.md`): no `CLAUDE.md`, tools or history. That prompt is
  generated from `idea.md`'s format table by `spike/contract`'s `render`.
- **The validator** is `hooks/contract.ts`, a port of the Rust crate in `spike/contract/`
  (the plugin runtime has no WebAssembly). Both run the same cases,
  `tests/contract-cases.ts`: `claude plugin test` in the plugin, `cargo test --test conform`
  in the crate. Changing one expected result fails both (*checked*). The Rust crate also builds to
  WebAssembly with workers-rs (`spike/worker/`), for celld later.

## Desktop window (Omarchy)

**Diagnosis (cm-dr0).** Weakest part: the plugin's state is visible only in a pane of the one
Claude Code session that captured it, and that pane competes with the agent for terminal width.
Force: the notes and the five fields need to be glanceable from anywhere on the desktop, without
giving up the in-session flow that works. Smallest move: a read-only window over the files the
plugin already writes, shown and hidden by a key. Nothing in the plugin changes except that its
events are now typed from one schema.

- **What it shows.** Every session under `~/.local/share/voice-brief/projects/*/notes/` that has a
  take or whose `claude` process is still running, newest first: its takes (sent, edited, cut off,
  deleted), and its placement (placing, failed, the proposal, or the accepted revision and whether
  it was handed off), each claim with the take its quote came from. It re-reads changed threads
  every second.
- **It drives recording, nothing else.** *Assumption, flagged (cm-vph):* Place, Accept and Hand off stay in Claude Code.
  Driving them from the window needs a command channel into the plugin (it polls files, so one is
  possible), and that protocol is not decided.
- **One schema for the thread.** The events are defined once, in Rust:
  [`desktop/thread/src/wire.rs`](desktop/thread/src/wire.rs). The window replays them with it, and
  the plugin's `ThreadEvent`, `ThreadLine`, `Claim` and `Field` types are generated from it with
  ts-rs into the top of `plugins/voice-notes/types/index.d.ts` (the plugin's types contract must
  be one self-contained file). `log()` in the plugin accepts only a `ThreadEvent`, so `tsc`
  rejects an event the window could not read. Regenerate with
  `cargo run -q -p vb-thread --bin thread-ts -- ../plugins/voice-notes/types/index.d.ts` from
  `desktop/`; `cargo test -p vb-thread --test typescript` fails while the committed file differs.
- **The fold** is `Thread::step(self, Event) -> Self` in `desktop/thread`, a crate with only
  serde, serde_json and ts-rs, so it builds and tests without GPUI (cm-hkg). Placement is a sum
  type (`Idle`, `Placing`, `Proposed`, `Failed`); accepting or rejecting outside `Proposed` changes
  nothing. A line that does not parse is counted, not fatal.
- **Push-to-talk is a state machine held as data** (cm-prk): `vb_talk::step(Talk, Event) ->
  (Talk, Option<Effect>)` in [`desktop/talk/`](desktop/talk/machine.md), framework-free so Kani can
  check the function itself. Holding Space in the window sends Claude Code's tap-mode Space to the
  terminal that owns the shown session (found by walking up from its `claude` pid to a Hyprland
  window); releasing it, or the window losing focus, sends the stop. The window learns a stream
  opened or closed from the proxy's `open/<stream>` marker, which now holds the client pid. The
  diagram in `desktop/talk/machine.md` is generated from `step` and drawn with frankenmermaid by
  `desktop/views` (SVG and terminal); its `cargo test` fails while the committed files differ.
- **Transitions are drawn as nodes.** frankenmermaid anchors an edge label at the middle of the
  edge's middle segment and never moves it (`fm_layout::edge_label_position`), so labels on this
  cyclic graph overlapped. Nodes are sized and separated by the layout, so each transition is a
  node of its own (a hexagon when it taps Space).
- **`desktop/views` is its own Cargo workspace.** Force: GPUI's tree turns on `log`'s
  `kv_serde`, which brings in `serde_fmt`, whose private `Error` converts both ways with
  `fmt::Error`; frankenmermaid's `egg` 0.11 then fails to infer a closure's error type
  (`dot.rs:229`), so any build unifying both trees breaks. Would reverse it: an `egg` release that
  names the type (`Ok::<(), fmt::Error>(())`).
- **Settling after a stop.** Force: the window sees the stream a poll late, so if the recorder stops
  by itself (Claude Code does, on silence) just before you release, the stop tap starts a new
  recording nobody holds. Kani found that interleaving. After a stop the machine waits in
  `Settling` for 1 s and stops any stream of that session that opens meanwhile. Proved with Kani
  (`desktop/talk/src/proofs.rs`): for every interleaving of 8 presses, releases, focus losses,
  stream events, recorder self-stops and other sessions' streams, releasing Space leaves the
  recorder off and the window idle. Assumption, flagged: a stream event reaches the window within
  the 1 s settle. Removing `Settling` makes that proof fail.
- **A focus blip is not a focus loss.** Hyprland's `send_key_state` to the terminal moves keyboard
  focus away from the window and back within a millisecond; the window treats focus as lost only if
  it is still lost 200 ms later. Seen live: without it, every recording stopped after 0.17 s.
- **F1 is Hyprland's.** Wayland gives an app no global keys, so the window (app id `voice-brief`)
  is sent to the special workspace `voicebrief` by a window rule, F1 toggles that workspace, and
  autostart launches the app at login: the Omarchy scratchpad idiom.
- **Build note.** `gpui-pre-macros` leaves a helper ungated in release builds while its module is
  gated on `debug_assertions`, so `desktop/Cargo.toml` turns debug assertions on for that one
  proc-macro in release, as disktree does.

## Where the data lives: per Git project

```
~/.local/share/voice-brief/projects/<git root, / replaced by ->/
  index.jsonl                 one row per voice stream: transcript, pid, project, clean close
  audio/<stream>.wav, .json   the exact frames sent (16 kHz mono linear16), and every server message
  open/<stream>               a stream in progress
  notes/<session start>/      one notes-pane session: thread.jsonl, each placement's files, prompt.md
```

A process outside any Git repo is filed under its working directory. *Assumption, flagged:* the
location (`$XDG_DATA_HOME/voice-brief`) is mine, chosen so captures survive `~/tmp` clean-ups
and stay out of git until audio in git is decided (Open questions). The central `~/briefs` repo
from the earlier decision is not created yet.

## The record: one append-only thread per idea

Each idea has one `thread.jsonl`, a list of typed events that is only ever appended to.
The events written today are defined in `desktop/thread/src/wire.rs` (Desktop window, above);
the list below is the planned set. Everything else about the idea (the five fields, the numbered revisions, where each sentence came
from, what is waiting for me) is computed from it. Prototype 3 keeps one thread per notes-pane
session; ideas as a unit of their own come later.

```
IdeaStarted         { title, project.path, project.remote, branch }
TakeRecorded        { take, stream, text, wav }
TakeSent            { take }
TranscriptCorrected { take, before, after }
TakeDeleted         { take, reason }
JobSubmitted        { job, base revision, takes in context }
ProposalMade        { kind: place | revise | review | question, claims: [said(spans) | added(reason)], asked }
JobFailed           { job, violations | error }
Accepted            { proposal }
Rejected            { proposal, reason }
Graded              { rubric_version, results }
HandedOff           { target: new-agent | paste | clipboard, revision }
ClosedNoChange      { reason }
```

- **Computing the state** is a pure transition `(state, event) -> (state, effects)`, matched
  exhaustively (cm-prk), framework-free (cm-hkg).
- **Transcripts stay correctable** without rewriting history: a correction appends
  `TranscriptCorrected`; a deletion appends `TakeDeleted`. Every claim citing a corrected or
  deleted take is re-checked and shown for review, never silently re-tagged (not built yet).
- **Would reverse this:** any operation that can only be done by changing or removing an earlier
  event.

### Job protocol

Model-checked in [`spec/tla/`](spec/tla/README.md) for the celld design: `JobSubmitted` before
sending, a job id created once, results deduplicated and acknowledged after the append, and accept
only for a proposal computed against the current revision with all its takes live. Prototype 3
runs placement synchronously inside the mod, so only the last rule applies there today.

## Grading

Code checks what code can (the validator in `spike/contract/`). The model judges the rest
against the definitions in `idea.md`:

| Field | Passes when |
|---|---|
| Weakest part | someone else could open the place named and check it; it cites evidence (churn, co-change, bug cluster, braid) |
| Force | two pressures in conflict, at least one citing something observed; not a symptom, not a guess |
| Smallest move | follows from the force: swapping in another pattern would break the text; fits in one commit |
| Preserves | says what must stay intact |
| Would reverse it | an observable condition: something that can be run or watched for (cm-gi3) |

"Nothing is weak; no structural change" is an outcome the app accepts; *assumption, flagged:*
the idea is then closed, not handed off.

## Local mode (later)

The earlier desktop design, kept for this mode:

- A background process, `vbd`, owns capture (`voxtype record start --file=<take>.txt --no-osd`
  with `pw-record` alongside for audio), keeps the model loaded for 10 minutes after a take (it
  owns that deadline itself; voxtype has no idle unload for its main model, *checked*), and is
  the only writer of the record.
- AI jobs run on celld under pi-durable with the job protocol above.
- Project resolution from Herdr's `session.snapshot`: the focused pane's Git root; else the single
  Git root among the focused workspace's panes; else the workspace label matched to
  `~/Projects/*/<label>`; else Unassigned. *Checked* on the live session (rule 2 was needed).
- Progress outside the window: an Omarchy bar widget, one notification when work is done, Herdr
  sidebar metadata.
- The mock-ups in `mockups/` show this design and predate both the five fields and the Claude
  Code mode.

Prototype 1 (in git history, `proto/vb_proto.py`) showed local transcription loses jargon
("reqwest", `~/.claude`) and needs a review step before a take enters the record.

## Decisions

The operator's words behind this design: [decisions.md](decisions.md).

## Open questions

1. celld target: a remote celld node (survives the desktop sleeping) or `celld dev` on the
   desktop.
2. Audio in git: the proxy captures every dictation's audio, kept untracked for now ("let's not
   track it in git yet"). Opus in the briefs repo, git-lfs, or outside git?
3. Where the record lives for good: the central `~/briefs` repo, per-project data as now, or
   both (data per project, briefs collected centrally)?
4. Should a question that hints at its own answer refute the spike's hypothesis, or only be
   reported?
