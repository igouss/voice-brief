# voice-brief desktop window

A window over the voice-notes plugin's sessions, for Omarchy (Hyprland). F1 slides it in and out.
It shows each session's takes and its placement: the proposal or accepted revision, every claim
with the take it quotes, and the questions where you said nothing. Hold Space to dictate: the
window has Claude Code record in the session it shows, and the take lands there as a note.

```
●  sync-app  proposal ready                         started 4 min ago   1 / 3
 3 takes          │ Proposal · 6.2 s · from t1 t2 t4
 t1  the weakest  │ Weakest part
     part is the  │ The sync module.
     sync module… │ said t1 “the weakest part is the sync module”
 t2  move the HTTP│ Preserves
     code behind… │ The sync behaviour seen by users stays the same.
     sent         │ added · a port move should not change behaviour; you did not say it
 t3  um testing   │ Would reverse it
     cut off      │ ?  What result after the move would show it was wrong?
 F1 hide  [ older  ] newer  0 latest
```

## Build and install

Needs Rust (edition 2024) and GPUI's Wayland build dependencies. From this folder:

```bash
cargo build --release
install -m755 target/release/voice-brief ~/.local/bin/voice-brief
```

## Hook it into Hyprland

In `~/.config/hypr/hyprland.lua`:

```lua
o.window("^(voice-brief)$", { float = true, center = true, size = { "(monitor_w*0.62)", "(monitor_h*0.72)" }, workspace = "special:voicebrief silent" })
```

In `~/.config/hypr/bindings.lua`:

```lua
o.bind("F1", "Voice brief", hl.dsp.workspace.toggle_special("voicebrief"))
```

In `~/.config/hypr/autostart.lua`:

```lua
o.launch_on_start("voice-brief")
```

Then `hyprctl reload`, and start it once now with `uwsm-app -- voice-brief`.

## Keys

| Key | Does |
|---|---|
| Space (hold) | talk: records through the shown session's Claude Code, which must be in tap mode (`"voice": { "mode": "tap" }` in the project's `.claude/settings.local.json`); the dictation becomes a note and never reaches the agent |
| F1 | show or hide (Hyprland) |
| `[` or ← | older session |
| `]` or → | newer session |
| `0` or Home | follow the newest session again |

## Layout

| Path | What |
|---|---|
| `talk/` | `vb-talk`: push-to-talk as a pure state machine, its generated diagram ([`machine.md`](talk/machine.md)) and its Kani proofs |
| `views/` | `vb-views`, its own workspace: regenerates the diagrams with [frankenmermaid](https://github.com/Dicklesworthstone/frankenmermaid) (`cargo run -q` there), and fails its tests when they are stale; `cargo run -q --bin preview -- out.png` rasterizes the diagram with resvg and the system's fonts |
| `src/talk.rs` | the adapters push-to-talk needs: settings, `/proc`, the proxy's open-stream markers, `hyprctl` |
| `thread/` | `vb-thread`: the thread event schema (`src/wire.rs`), the fold that replays it, and `thread-ts`, which writes the plugin's TypeScript types from the schema |
| `src/store.rs` | finds sessions under `$XDG_DATA_HOME/voice-brief/projects` and re-reads changed threads |
| `src/view.rs` | the window |

`cargo test` in `views/` checks the generated diagrams; `cargo kani` in `talk/` proves the push-to-talk properties (about 2 minutes; Kani 0.68:
`cargo install --locked kani-verifier && cargo kani setup`). `cargo test --workspace` runs the fold against a real (anonymised) session and checks that
`../plugins/voice-notes/types/index.d.ts` still begins with the generated types. Why it is shaped
this way: [`../design.md`](../design.md), "Desktop window".
