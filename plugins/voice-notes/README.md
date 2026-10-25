# voice-notes

A Claude Code plugin: dictate an idea with `/voice`, and get a five-field task prompt (Weakest
part, Force, Smallest move, Preserves, Would reverse it). Every claim is tagged `said` with your
exact words, or `added` with a reason. A field you left empty comes back as a question.

Part of [voice-brief](https://github.com/igouss/voice-brief); the design and the security notes
are there.

## Install

```
/plugin install voice-notes --marketplace igouss/voice-brief
```

Then, once per project, in Claude Code at the project's root:

```
/vb-setup
```

and restart Claude Code there.

`/vb-setup` needs Linux with systemd user services and mitmproxy (`uv tool install mitmproxy`). It
does three things:

- installs `vb-voice-proxy.service`, a local mitmproxy on 127.0.0.1:8897 that decrypts only
  `api.anthropic.com` and saves Claude Code's voice streams;
- copies the proxy addon to `~/.local/share/voice-brief/`;
- writes `HTTPS_PROXY` and `NODE_EXTRA_CA_CERTS` into the project's `.claude/settings.local.json`.

The proxy decrypts every request a proxied session makes to Anthropic, OAuth token included. It
writes only voice streams to disk.

## Use

`/voice` and dictate. Each dictation becomes a note in the Voice notes pane as soon as it is
transcribed; Enter only decides whether the agent also gets it.

**Tap mode** (`"voice": { "mode": "tap" }` in the project's `.claude/settings.local.json`): Space
starts recording and Space again stops it, and Claude Code then submits the transcript by itself.
The plugin drops that submission before it reaches the agent ("Kept as voice note tN; not sent to
the agent") and clears the prompt box. Only a prompt that is exactly a dictation this session just
captured is dropped; anything typed is sent. Tap mode is what lets the desktop window start and
stop recording with one key each.

| Command | Does |
|---|---|
| `/vb`, `/vb-min`, `/vb-max` | open, minimize to one line above the prompt, maximize |
| `/vb-place` | place every live note into the five fields |
| `/vb-accept`, `/vb-reject` | accept or reject the proposal |
| `/vb-handoff` | copy the accepted prompt to the clipboard and write `prompt.md` |
| `/vb-delete tN`, `/vb-fix tN <text>` | drop a note, or correct its transcript |
| `/vb-setup` | install the proxy and this project's settings |

Placement is one call through Claude Code's own model API (`claude-opus-5-5`, effort medium),
with only the plugin's system prompt: no `CLAUDE.md`, tools or history. A validator in the
plugin rejects any reply whose quoted words are not in your notes.

Data, per Git project: `~/.local/share/voice-brief/projects/<git root with / as ->/`.

## Checks

```bash
claude plugin validate plugins/voice-notes
claude plugin test plugins/voice-notes
(cd plugins/voice-notes/setup && ~/.local/share/uv/tools/mitmproxy/bin/python -m unittest test_vb_mitm)
```

The plugin's `ThreadEvent`, `ThreadLine`, `Claim` and `Field` types at the top of
`types/index.d.ts` are generated from `desktop/thread/src/wire.rs`; `cargo test --workspace` in
`desktop/` fails when they drift.
