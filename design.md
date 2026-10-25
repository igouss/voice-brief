# voice-brief — design

Develop an idea by voice and keyboard, with AI help, into a task prompt for an agent, and hand
the prompt to an agent in the project you are working on. The problem, the prompt format and its
lineage are in [`idea.md`](idea.md). Mock-ups: [`mockups/01-workspace.html`](mockups/01-workspace.html).

Status: design. Nothing is built. Claims marked *checked* were run on this machine on
2026-10-25; everything else is intended behaviour.

## Diagnosis (cm-dr0)

Nothing exists, so the weakest part is the riskiest assumption: that the model can place my
dictated words into the five fields (weakest part, force, smallest move, preserves, would
reverse it) while
separating what I said from what it adds, and that it asks instead of inventing a field I have not
filled. The first move is a spike on dictations of real ideas, before any UI.

## The unit: an idea is a task prompt

| Field | What it says |
|---|---|
| Weakest part | Where the current whole is weakest: a specific component, file or step |
| Force | The pressure that is unresolved there |
| Smallest move | The one change that resolves the force and leaves the rest intact |
| Preserves | What the change must leave intact |
| Would reverse it | The observable result that would show the change was wrong |

Weakest part, force and smallest move come from cm-dr0; preserves and would reverse it from
cm-e2y. Words I have said that are not yet placed in a field sit in **Unsorted**. The prompt handed to
an agent is the five fields plus my words verbatim with the date (cm-6nu).

## Forces and the cuts they earn (cm-xrl)

| Force | Cut |
|---|---|
| The mic and the speech model are local; the window can be hidden or closed while work continues | `vbd`, a background process on the desktop, separate from the `vb` TUI |
| AI jobs are slow and must survive the desktop sleeping | AI runs on celld under pi-durable, not in `vbd` |
| History, deletions and revisions must be inspectable and permanent | one append-only thread per idea is the record, carried by git; celld keeps no idea state |
| The bar, Herdr and the TUI all show the same progress | one state file written by `vbd`, read by all three |

No other boundary is earned yet. In particular there is no drafter port: Claude through celld is
the only adapter.

## Components

```
desktop                                                        celld node
┌─────────────── vbd (Rust, systemd user unit) ──────────────┐  HTTP+WS   ┌──────────────────────────┐
│ capture: voxtype record --file + pw-record                 │ ─────────► │ IdeaCell, one per idea   │
│ model lifetime: start voxtype, unload 10 min after a take  │  jobs with │   pi-durable harness     │
│ project resolution: herdr session.snapshot + events        │ idempotency│ AccountCell (Claude      │
│ job queue: submit, follow, retry; results to the idea      │    keys    │   OAuth, shared with     │
│ git writer: ~/briefs                                       │ ◄───────── │   pi-world)              │
│ state: $XDG_RUNTIME_DIR/voice-brief/state.json             │   events   └──────────────────────────┘
│ herdr: sidebar metadata, notifications, agent hand-off     │
└──────▲───────────────────────────────▲─────────────────────┘
       │ unix socket                   │ watches state.json
┌──────┴──────────┐            ┌───────┴────────────────────────┐
│ vb (ratatui TUI)│            │ elendal.voice-brief bar widget │
│ floating window │            │ (Omarchy shell plugin, QML)    │
└─────────────────┘            └────────────────────────────────┘
```

- **`vbd`** owns everything that must outlive the window. It starts on login (or on the first
  toggle) and is the only writer of git and of the state file.
- **`vb`** is a client of `vbd` over a unix socket. Closing or hiding it changes nothing in
  progress.
- **The domain** (the event types, the transition function that computes an idea's state,
  rubric checks, assembling the prompt) is a framework-free module with no I/O (cm-hkg).
- **celld** runs only the AI jobs. Each job carries everything it needs (the thread so far,
  the transcripts it refers to, rubric, my instruction) and returns `ProposalMade` events, so the cell can be wiped without losing anything.
  `AccountCell` is used for the second time here, which earns extracting it from pi-world into a
  shared package.

## The record: one append-only thread per idea

The conversation is the record. Each idea has one `thread.jsonl`, a list of typed events that is
only ever appended to. Everything else about the idea (the five fields, the numbered revisions,
where each sentence came from, what is waiting for me) is computed from it.

```
~/briefs/<idea>/thread.jsonl        the record: typed events, append-only
~/briefs/<idea>/takes/0012.opus     audio (Opus: ~50 KB per 30 s instead of ~1 MB of WAV)
~/briefs/<idea>/takes/0012.toml     transcript, started, duration, model
~/briefs/<idea>/prompt.md           generated view of the five fields; never edited by hand
~/briefs/rubrics/task-v1.toml       the rubric, generated from idea.md's format table
```

### Events

```
IdeaStarted         { title, project.path, project.remote, branch, herdr workspace label }
TakeRecorded        { take, target: field | unsorted, mode: append | revise | answer, transcript_hash }
TranscriptCorrected { take, transcript_hash, reason }
TakeDeleted         { take, reason }
JobSubmitted        { job, base revision, transcripts in context }
Typed               { field, at, text }
ProposalMade        { kind: place | revise | review | question, claims: [said(take|typed, span) | added(reason)] }
Accepted            { proposal }
Rejected            { proposal, reason }
Asked               { field, question }
Graded              { rubric_version, results }
HandedOff           { target: new-agent | paste | clipboard | inbox, revision }
ClosedNoChange      { reason }
```

- **Computing the state** is a pure transition `(state, event) -> (state, effects)`, matched
  exhaustively on every event type (cm-prk), in the framework-free domain (cm-hkg). The prompt,
  the history view and the AI's context are all outputs of it.
- **Revisions** are numbered by counting the events that change a field (`Accepted`, `Typed`).
  The history view is the thread itself, rendered.
- **A sentence's source** is a take or a `Typed` event: what I type is my words too. Editing a
  claim the AI tagged `added` makes the edited claim mine.
- **Transcripts stay out of the thread**, so correcting or deleting one never rewrites it. The
  thread holds a hash of each transcript version, so a replay can check it has the right file.
  - **Correcting a transcript** (the speech-to-text got my words wrong, or I rephrase what I
    meant): the corrected text replaces `takes/0012.toml`, and `TranscriptCorrected` records the
    new hash and why. The old text stays in git.
  - **Deleting a take** (the transcript is wrong and not worth fixing): `TakeDeleted` is
    appended, the audio and transcript are removed with `git rm`, and the take leaves the AI's
    context. It stays in git history.
  - After either, every claim citing that take is re-checked: a `said` span that is no longer in
    the transcript, or whose take is deleted, is marked unsupported and shown to me for review.
    Nothing is silently re-tagged.
- **git** carries the thread and the takes and is the history; a commit follows each event or
  each accepted batch. Herdr ids (`w1`, `w1:p1`) are never stored: they do not survive a server
  restart.
- **Would reverse this:** any operation that can only be done by changing or removing an earlier
  event. The experiment bead checks the mock-up scenarios for one.

### Job protocol

Model-checked in [`spec/tla/`](spec/tla/README.md); `spec/tla/check.sh` passes only if the
design passes and each rule's removal is caught.

- `JobSubmitted` is appended before the job is sent to celld, and `vbd` re-sends every submitted
  job without a result when it restarts.
- celld creates a job id once; a repeated submit is ignored.
- A result for a job that already has one is acknowledged, not appended; the ack is sent only
  after the result is in the thread.
- A proposal can be accepted only if it was computed against the current revision and every
  transcript it used is still live; otherwise it is shown as stale.
- `vbd` runs on one machine and is the only writer of `~/briefs`.

## Speech-to-text and the model's lifetime

voxtype 1.1.1 has no idle timeout for its main model (*checked*: only
`cold_model_timeout_secs`, for secondary models, and `on_demand_loading`, which unloads at once).
So `vbd` controls the daemon:

- Toggling the app on starts `voxtype.service` (installed, currently disabled).
- Every take re-arms `systemd-run --user --on-active=600 --unit=voice-brief-unload systemctl
  --user stop voxtype`. An explicit unload key stops it at once.
- Each take runs `voxtype record start --file=<take>.txt --no-osd`, so voxtype never types into
  the focused window, and `pw-record` alongside it for the audio, which the daemon does not keep.
  *Not yet checked:* two readers on one mic, and how well their start and stop line up.
- Stages come from voxtype's state file (`/run/user/1000/voxtype/state`: idle, recording,
  transcribing) plus `vbd`'s own (saving audio, applying, committing). voxtype gives no
  percentage; an estimate from recent takes on this machine is shown as an estimate.

## Project resolution (Herdr)

When a take starts, `vbd` reads `session.snapshot` and resolves the project in this order:

1. the focused pane's git root;
2. else the single git root among the panes of the focused workspace;
3. else the workspace label matched against `~/Projects/*/<label>`;
4. else the Unassigned inbox, with a picker.

*Checked* on the live session: the focused workspace was `w9` "unikernel", the focused pane
`w9:p8` sat in `/home/elendal`, and only `w9:p1` was in a repo (`~/Projects/mine/unikernel`).
Rule 1 alone would have failed; rule 2 resolves it. Herdr's focus is its own, so it is still
right while the voice-brief window or a browser has Hyprland's focus. An SSH pane reports its
local directory, not the remote one.

`events.subscribe` keeps the TUI header and the idea list filtered to the current project. A
Herdr plugin action, "voice-brief: note here", passes the invocation context (`workspace_cwd`,
`focused_pane_cwd`, `worktree`) directly.

## Flows

A take goes to the selected field, or to Unsorted.

| Action | What happens | AI | Result |
|---|---|---|---|
| Append | Transcript inserted at the editor cursor in the selected field | no | Unsaved edit |
| Edit | Built-in editor (`tui-textarea`) | no | Unsaved edit |
| Save | Commit the edited text | no | Next revision |
| Sort | Unsorted words → proposed placements into the fields, and a question for each field still empty | yes | Accept per placement |
| Revise | New take + current field → a proposed revision of that field, shown as a diff | yes | Accept = next revision; a reject is a `Rejected` event with my reason |
| AI review | A visible, editable review prompt (template plus my instruction, typed or spoken) → proposals | yes | Accept per proposal |
| Grade | Rubric checklist per field: pass / partial / missing, one suggestion per gap | partly | Shown beside the prompt; re-graded after each revision |
| Ask | The AI asks about the weakest field; I answer by voice or keyboard; the answer is a take | yes | Proposals for that field |
| New idea, idea list | `n` starts an idea; the list shows past ideas with status, filtered to the current project | no | — |
| Correct transcript | Fix what speech-to-text got wrong, or rephrase what I meant | no | `TranscriptCorrected`; claims citing the take are re-checked |
| Delete take | `TakeDeleted`, then `git rm` the take | no | Gone from the tree and the AI context; kept in history; claims citing it marked unsupported |

Every proposal is tagged `said` (with the transcript it comes from) or `added` (no transcript
supports it). The AI never fills an empty field with a guess: it asks. Nothing is written without
my accept.

**Grading follows cm-8gv.** Code checks what code can: every field is non-empty, and my words
are quoted verbatim with the date. The model judges the rest against `rubrics/task-v1.toml`,
which it cannot change:

| Field | Passes when |
|---|---|
| Weakest part | someone else could open the place named and check it; it cites evidence (churn, co-change, bug cluster, braid) |
| Force | two pressures in conflict, at least one citing something observed (a bug, a build artifact, a change that hurt); not a symptom, not a guess |
| Smallest move | follows from the force: swapping in another pattern would break the text; fits in one commit |
| Preserves | says what must stay intact |
| Would reverse it | an observable condition: something that can be run or watched for (cm-gi3) |
| all | the text would change if the evidence changed |

The definitions are in `idea.md`'s format table; the rubric file is generated from it, not
written separately (cm-eds). "Nothing is weak; no structural change" is an outcome the app
accepts; *assumption, flagged:* the idea is then closed, not handed off.

## Hand-off

The current revision of the prompt goes to an agent in two ways:

- **New agent:** in the idea's project workspace, `pane split --cwd <project> --no-focus`, then
  `agent start idea-<id> --kind claude|pi|codex`, then `agent prompt`. The agent's state
  (working, blocked, done) is followed through `events.subscribe` and shown on the idea.
- **Paste into the selected agent:** the target defaults to Herdr's focused agent and can be
  changed from a list of agents in the project's workspace. *Assumption, to confirm:* the text is
  pasted without pressing Enter, so I submit it myself. *Not yet checked:* that
  `pane send-text` uses bracketed paste for multi-line text.

Also: copy to the clipboard, or save to `inbox/`.

## Seeing progress while hidden

Hyprland has no minimize: hiding (`q`) moves the window to a special workspace and the toggle key
brings it back. `vbd` keeps working either way. Progress shows in three places, all from
`state.json`:

- **Bar widget** `elendal.voice-brief`, an Omarchy shell bar-widget plugin built like
  `elendal.audio`, placed next to `omarchy.agents`: model unloaded / loaded with countdown /
  recording to a project / working with the stage / done with a count.
- **Notification** once, when all work for an idea is done.
- **Herdr sidebar:** `workspace report-metadata --source voice-brief` on the project's workspace
  shows its open idea and anything waiting; a TTL clears it if `vbd` stops.

The recording overlay (waveform, then transcription stages) appears when a take starts and closes
itself when the transcript lands.

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

## Open questions

1. celld target: the oracle-arm node (survives the desktop sleeping; pi-world's routes have no
   authentication on the tailnet) or `celld dev` on the desktop.
2. Audio in git: Opus in a plain private repo, or git-lfs.
3. Engine: parakeet-tdt-0.6b-v2 (current default) or whisper large-v3-turbo, measured on my own
   takes.
4. Whether a separate key also does ordinary dictation into any window while the model is loaded.
