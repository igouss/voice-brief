# voice-brief — design

Speak ideas into a brief, refine them by voice and keyboard with AI help, and hand the
finished prompt to an agent in the project you are working on. The problem and the
motivation are in [`idea.md`](idea.md); the golden cases are in [`examples.md`](examples.md).
Mock-ups: [`mockups/01-workspace.html`](mockups/01-workspace.html).

Status: design. Nothing is built. Claims marked *checked* were run on this machine on
2026-10-25; everything else is intended behaviour.

## Diagnosis (cm-dr0)

Nothing exists, so the weakest part is the riskiest assumption: that the model can revise,
review and grade an idea while separating what I said from what it adds. The first move is a
spike on the four golden cases in `examples.md`, before any UI. The spike is refuted if a
revision drops what I meant or if added claims (cm-nr1's CI diffing, cm-27o's calibration) come
back unflagged.

## Forces and the cuts they earn (cm-xrl)

| Force | Cut |
|---|---|
| The mic and the speech model are local; the window can be hidden or closed while work continues | `vbd`, a background process on the desktop, separate from the `vb` TUI |
| AI jobs are slow and must survive the desktop sleeping | AI runs on celld under pi-durable, not in `vbd` |
| History, deletions and revisions must be inspectable and permanent | git is the record; celld keeps no brief state |
| The bar, Herdr and the TUI all show the same progress | one state file written by `vbd`, read by all three |

No other boundary is earned yet. In particular there is no drafter port: Claude through celld is
the only adapter.

## Components

```
desktop                                                        celld node
┌─────────────── vbd (Rust, systemd user unit) ──────────────┐  HTTP+WS   ┌──────────────────────────┐
│ capture: voxtype record --file + pw-record                 │ ─────────► │ BriefCell, one per brief │
│ model lifetime: start voxtype, unload 10 min after a take  │  jobs with │   pi-durable harness     │
│ project resolution: herdr session.snapshot + events        │ idempotency│ AccountCell (Claude      │
│ job queue: submit, follow, retry; results to the brief     │    keys    │   OAuth, shared with     │
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
- **The domain** (briefs, ideas, revisions, takes, rubric checks, id minting, assembling the final
  prompt) is a framework-free module with no I/O, tested against `examples.md` (cm-hkg).
- **celld** runs only the AI jobs. Each job carries everything it needs (current revision,
  transcripts in use, rubric, my instruction), so the cell can be wiped without losing anything.
  `AccountCell` is used for the second time here, which earns extracting it from pi-world into a
  shared package.

## The record: `~/briefs` (one central git repo)

```
~/briefs/<brief>/brief.toml          title, kind (definition | task), status, project.path,
                                     project.remote, branch, herdr workspace label (display only)
~/briefs/<brief>/ideas/03-<slug>.md  current text; front matter: revision = 7
~/briefs/<brief>/takes/0012.opus     audio (Opus: ~50 KB per 30 s instead of ~1 MB of WAV)
~/briefs/<brief>/takes/0012.toml     idea, mode, started, duration, model, project, transcript
~/briefs/<brief>/reviews/0004.json   prompt sent, proposals, my decision and reason for each
~/briefs/<brief>/grades/0009.json    rubric version, item results, suggestions
~/briefs/<brief>/task.md             the final prompt, with its own revision number
```

- **Revisions are numbered per idea and per task.** Each accepted change is one commit with
  trailers `Revision: idea-3/r7` and `Takes: 0011, 0012`. The history view is `git log` on the
  file with those trailers rendered.
- **Deleting a take** is `git rm` of its audio and transcript. It leaves the tree and the AI
  context, and stays in history; old revisions still name it in their trailers.
- Herdr ids (`w1`, `w1:p1`) are never stored: they do not survive a server restart.

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

`events.subscribe` keeps the TUI header and the brief list filtered to the current project. A
Herdr plugin action, "voice-brief: note here", passes the invocation context (`workspace_cwd`,
`focused_pane_cwd`, `worktree`) directly.

## Flows

| Action | What happens | AI | Result |
|---|---|---|---|
| Append | Transcript inserted at the editor cursor | no | Unsaved edit |
| Edit | Built-in editor (`tui-textarea`) | no | Unsaved edit |
| Save | Commit the edited text | no | Next revision |
| Revise | New take + current revision → a proposed revision, shown as a diff | yes | Accept = next revision; reject kept in `reviews/` |
| AI review | A visible, editable review prompt (template plus my instruction, typed or spoken) → proposals | yes | Accept per proposal |
| Grade | Rubric checklist: pass / partial / missing, a score, one suggestion per gap | partly | Shown beside the idea; re-graded after each revision |
| Task builder | A durable conversation per brief: the AI asks, I answer by voice or keyboard, the task advances by revisions | yes | `task.md`, graded |
| Answer by voice | In a review or the task builder, a take becomes my reply and is kept as a take | — | Both |
| New idea, brief list | `n` adds an idea; the brief list shows past briefs with status, filtered to the current project | no | — |
| Delete take | `git rm` the take | no | Gone from the tree and the AI context; kept in history |

Every proposal is tagged `said` (with the transcript it comes from) or `added` (no transcript
supports it). Nothing is written without my accept.

**Grading follows cm-8gv:** items code can check are checked by code (rule has a minted id, ids
are unique, a gate is named, my words are in the Operator decision verbatim with the date). Only
judgement items go to the model (the force is stated; the task says what, not how, per cm-v6i).
The rubric is a versioned file per kind in `~/briefs/rubrics/`, so the model cannot redefine it.

## Hand-off

The current revision of the task prompt goes to an agent in two ways:

- **New agent:** in the brief's project workspace, `pane split --cwd <project> --no-focus`, then
  `agent start brief-<id> --kind claude|pi|codex`, then `agent prompt`. The agent's state
  (working, blocked, done) is followed through `events.subscribe` and shown on the brief.
- **Paste into the selected agent:** the target defaults to Herdr's focused agent and can be
  changed from a list of agents in the project's workspace. *Assumption, to confirm:* the text is pasted without pressing
  Enter, so I submit it myself. *Not yet checked:* that `pane send-text` uses bracketed paste for
  multi-line text.

The other targets remain: copy to the clipboard, save to `inbox/tasks/`, and for a definition, a
patch to `CLAUDE.md` in the dotfiles repo, committed only after I confirm the diff.

## Seeing progress while hidden

Hyprland has no minimize: hiding (`q`) moves the window to a special workspace and the toggle key
brings it back. `vbd` keeps working either way. Progress shows in three places, all from
`state.json`:

- **Bar widget** `elendal.voice-brief`, an Omarchy shell bar-widget plugin built like
  `elendal.audio`, placed next to `omarchy.agents`: model unloaded / loaded with countdown /
  recording to a project / working with the stage / done with a count.
- **Notification** once, when all work for a brief is done.
- **Herdr sidebar:** `workspace report-metadata --source voice-brief` on the project's workspace
  shows its open brief and anything waiting; a TTL clears it if `vbd` stops.

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

## Open questions

1. celld target: the oracle-arm node (survives the desktop sleeping; pi-world's routes have no
   authentication on the tailnet) or `celld dev` on the desktop.
2. Audio in git: Opus in a plain private repo, or git-lfs.
3. The first rubric for each kind: drafted from `CLAUDE.md`, cm-v6i and cm-6nu for me to edit, or
   written by me.
4. Engine: parakeet-tdt-0.6b-v2 (current default) or whisper large-v3-turbo, measured on my own
   takes.
5. Whether a separate key also does ordinary dictation into any window while the model is loaded.
