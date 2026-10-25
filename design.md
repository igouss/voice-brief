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

The design changed under it on 2026-10-25 (see Decisions). Speech-to-text and the interface no
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
Everything else about the idea (the five fields, the numbered revisions, where each sentence came
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

Operator words, 2026-10-25:

- "I want to use the model when I use the app … I want the model to remain in the memory for a
  while … say for 10 min."
- "Append (at the current cursor position), revise - take a new recording and current version and
  showed a poposed revision, and a way to accept it. I should be able to edit it in the app, no
  need for the external editor."
- "History yes, in git." "I do want to be able to delete some transcripts and voice recording
  (they stay in git)."
- "since this would be a long running process, let's use pi-durable and celld I have."
- "I think one central repo." "Showing briefs in Herdr's sidebar - that would be cook, yeah!"
- "background process is not probelm."
- "I should be able to start a new agent for the project in herder with the current revision of
  the prompt, and I should be able to paste to the currently selected agent."
- "OK to make changes, those are my personal set-up projects."
- "this is a dictation app, to develop my idea using my microphone and AI in the format that I
  want: Weakest part, Force, Preserves, Would reverse it, format."
- "building a task prompt for the agent is the idea."
- "Weakest part, force, smallest move come from cm-dr0 … Preserves, would reverse it come from
  cm-e2y … Smallest move stays."
- "OK for one thread.jsonl per idea." "Regarding deletion, sometimes the idea needs a rephrasing,
  or to be deleted because transcipt is wrong."
- "one machine only, install tools locally" "ok model it now"
- "Yeah, transcription is weak, I would need to view and approve it before it enters into the
  system."
- "I think this workflow actually might work well. We don't need the dictation up from the vox we
  can install, not install, just use the dictation that the Claude Code provides. It seems to be
  pretty accurate and fast."
- Dictated as corpus take t3: "The application itself should or still support recording and
  transcribing locally through the vox when I'm not running it from the claude code instance."
  Then, asked whether to record it: "yes, add it to design.md".
- "to be a dual mode application. It should support Claude Code and it should support recording
  when it's running outside the Claude Code. For now, concentrate on the Claude Code version."
- "it seems stupid to have a rolling buffer when we could just intercept the web socket and
  capture the exact data that we capture."
- "TypeScript is fine but I don't like it as a language, WASM is fine too … Maybe we can do rust
  to wasm?" "use the standard workers-rs toolchain"
- "I don't want to send the dictated text to the model when I don't want to, and if I do want to
  send the dictated text, I will just press enter … For now, a preferred mode where I just open
  Claude and use its voice … for this application to populate voice notes."
- "Maybe we could just use Claude mods and this void capture and processing, and we just do it
  from the inside Claude code. Maybe we don't need to write an application right now. That would
  be a follow-up possibility. Let's concentrate right now on a prototype that uses Claude code
  mods and brings some kind of user interface for the application that we're building."
- "The voice proxy runs inside whichever pane first started it, so closing that pane can cut
  every proxied Claude session off at once. Fix that please. And let's update the documents to
  reflect this pivot. Let's organize our voice notes and everything else that we generate so that
  it's per Git project, so that we could safely run proxy the Claude code."
- "Okay, package this as a Claude Code plugin." Asked how to run the validator and set up the
  proxy: "Port to TypeScript", "/vb-setup command". Then: "the proper rewrite. Don't spawn using
  command line. Use a built-in claude code facilities to do this."

## Open questions

1. celld target: a remote celld node (survives the desktop sleeping) or `celld dev` on the
   desktop.
2. Audio in git: the proxy captures every dictation's audio, kept untracked for now ("let's not
   track it in git yet"). Opus in the briefs repo, git-lfs, or outside git?
3. Where the record lives for good: the central `~/briefs` repo, per-project data as now, or
   both (data per project, briefs collected centrally)?
4. Should a question that hints at its own answer refute the spike's hypothesis, or only be
   reported?
