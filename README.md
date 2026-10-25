# voice-brief

Talk through an idea; get a task prompt a coding agent can act on, built from your own words.

voice-brief turns dictation into a five-field prompt: **Weakest part, Force, Smallest move,
Preserves, Would reverse it**. Every claim in it is tagged `said` (with the exact words it came
from) or `added` (with the AI's reason). A field you left empty comes back as a question, not a
guess.

> **Status: prototype, for one person's machine.** Linux, systemd user services, Claude Code
> 2.1.310 with `/voice` (a claude.ai login). Paths, ports and the workflow are tuned to the
> author's setup. Read [Security](#security) before running it.

## The problem

Ideas for work arrive as speech. Dictating a rough paragraph into an agent works, but:

- the raw words are lost once the agent paraphrases them;
- the agent fills the gaps with plausible guesses, and nothing marks which parts were yours;
- an idea needs several passes, and nothing holds it between them.

voice-brief keeps every dictation (text and audio), places it into a fixed format with the source
of each sentence visible, and asks where you said nothing. The format and why it is shaped this
way: [`idea.md`](idea.md).

## What it looks like (illustrative)

```
Notes (3)  /home/you/Projects/some-repo
t1 the weakest part is the sync module, every time I change the HTTP client the desktop build breaks
t2 ↗ move the HTTP code behind a port so the desktop build doesn't pull in reqwest
t3 the desktop build must still pass
[ Place ]  Accept  Reject

Proposal from t1, t2, t3
Weakest part
  The sync module.
    said t1
Force
  Changing the HTTP client breaks the desktop build.
    said t1
Smallest move
  Move the HTTP code behind a port so the desktop build does not depend on it.
    said t2
Preserves
  The desktop build must still pass.
    said t3
Would reverse it
  ? What result after the move would show it was wrong and should be undone?
```

`↗` marks a note you also sent to the agent with Enter. Hand off copies the accepted prompt to the
clipboard.

## How it works

```
claude (at the repo root)                    vb-voice-proxy.service (systemd user unit)
┌───────────────────────────────────┐        ┌────────────────────────────────────────┐
│ /voice dictation ─ voice_stream ──┼───────►│ mitmdump 127.0.0.1:8897                │──► api.anthropic.com
│                                   │ HTTPS_ │ decrypts api.anthropic.com only        │
│ voice-notes mod (Claude Code mod) │ PROXY  │ vb_mitm.py: each voice stream → WAV,   │
│  reads its project's index.jsonl ◄┼────────┤ final transcript, client pid, filed    │
│  pane: notes, Place/Accept/Hand off│        │ under the client's Git root            │
│  Place → isolated claude -p        │        └────────────────────────────────────────┘
│        → Rust validator            │
└───────────────────────────────────┘
```

1. **Capture.** Claude Code transcribes `/voice` dictation in Anthropic's cloud and keeps no
   audio. A local mitmproxy, running as a systemd user service, saves each voice stream's exact
   audio and Anthropic's transcript. It ties each stream to the `claude` process that opened it
   and that process's Git project.
2. **Notes.** The `voice-notes` mod turns every dictation from its own session into a note the
   moment it is transcribed. Enter is optional: it only decides whether the agent in that session
   also gets the text.
3. **Placement.** Place sends all live notes to an isolated `claude -p` that sees only an
   explicit prompt (no `CLAUDE.md`, memory, skills or tools). The field definitions are read from
   `idea.md`. A Rust validator rejects any reply whose quoted words are not in the transcript, or
   whose field has both claims and a question or neither.
4. **Hand-off.** Accept, then Hand off: the five fields as Markdown, on the clipboard and in
   `prompt.md`.

The full design, with what was checked and what is assumed: [`design.md`](design.md).

## Setup

Requirements: Linux with systemd user services, Claude Code with `/voice`, [`uv`](https://docs.astral.sh/uv/),
`jq`, and a Rust toolchain.

```bash
git clone https://github.com/igouss/voice-brief.git
cd voice-brief
uv tool install mitmproxy
proto/install.sh
```

`proto/install.sh`:

- creates the mitmproxy CA in `~/.mitmproxy/` if it is missing;
- installs and starts `vb-voice-proxy.service` (from `proto/vb-voice-proxy.service.in`, with
  this repo's path);
- writes `.claude/settings.local.json` (gitignored) with the CA path and this repo as the
  `voice-brief` plugin marketplace;
- builds the contract tools in `spike/contract`.

## Use

```bash
claude          # at the repo root; accept the project's marketplace and plugin when asked
```

Then:

1. `/voice` and dictate. Each dictation shows up in the **Voice notes** pane, with a "captured"
   line above the prompt.
2. Answer the questions by dictating again.
3. **Place**, **Accept**, **Hand off**.

| Command | Does |
|---|---|
| `/vb` | open the Voice notes pane |
| `/vb-place` | place every live note into the five fields |
| `/vb-accept`, `/vb-reject` | accept or reject the proposal |
| `/vb-handoff` | copy the accepted prompt to the clipboard and write `prompt.md` |
| `/vb-delete t3` | drop a note from placement |
| `/vb-fix t3 <text>` | correct a note's transcript |

In another repository, `proto/vb-claude.sh --brief` starts a session with the proxy and the mod
loaded.

## Where the data lives

Per Git project, outside the repository:

```
~/.local/share/voice-brief/projects/<git root with / replaced by ->/
  index.jsonl              one row per voice stream: transcript, pid, project, clean close
  audio/<stream>.wav       the exact 16 kHz mono frames Claude Code sent
  audio/<stream>.json      every message Anthropic sent back
  notes/<session>/         thread.jsonl (append-only events), placements, prompt.md
```

## Security

- **The proxy decrypts the whole conversation of every session that uses it**, including your
  OAuth token in request headers. It writes only voice streams to disk, binds to 127.0.0.1, and
  decrypts only `api.anthropic.com`; other hosts pass through untouched.
- **`~/.mitmproxy/mitmproxy-ca.pem` is a certificate authority.** Anyone holding that key can
  impersonate `api.anthropic.com` to a session that trusts it. Only sessions with
  `NODE_EXTRA_CA_CERTS` pointing at it trust it.
- **The voice endpoint is undocumented.** `/api/ws/speech_to_text/voice_stream` and its messages
  (`TranscriptText`, `TranscriptEndpoint`, `CloseStream`) can change in any Claude Code release,
  and then no dictation becomes a note.

## Troubleshooting

| Symptom | Cause | Fix |
|---|---|---|
| Every session in the repo hangs on its first request | the proxy service is down | `systemctl --user status vb-voice-proxy`, then `systemctl --user restart vb-voice-proxy` |
| `SSL certificate verification failed` | the CA path is wrong or unexpanded | rerun `proto/install.sh`; settings `env` does not expand `$HOME` or `~` |
| No Voice notes pane, no `/vb` commands | `claude` started in a subdirectory, or the plugin was not trusted | start `claude` at the repo root and accept the marketplace; `/reload-plugins` after editing the mod |
| Dictations never become notes | no voice stream reached the proxy, or the stream format changed | check `~/.local/share/voice-brief/projects/*/index.jsonl` for a new row with your session's pid |
| Place says "model call failed" | the contract tools are not built, or `claude -p` failed | `cargo build --bins --manifest-path spike/contract/Cargo.toml`; look in `notes/<session>/place-*/` |

## Limitations

- One machine, one person. No packaging, no multi-user story.
- Linux only: the proxy finds the client process through `/proc`.
- The local mode (transcribing with voxtype on this machine) is designed but not built.
- The checker that scores placement against a confirmed annotation, and the full spike run, are
  not done yet (`spike/`); the claim that placement invents nothing is not yet measured.
- Correcting or deleting a note does not yet re-check accepted claims that cite it.
- Every session started at this repo's root is a notes session (`VB_BRIEF=1` in
  `.claude/settings.json`).

## Repository

| Path | What |
|---|---|
| [`idea.md`](idea.md) | the problem and the five-field format, the single source of the field definitions |
| [`design.md`](design.md) | the design, decisions in the operator's words, open questions |
| [`proto/`](proto/NOTES.md) | the proxy addon and service, the `voice-notes` mod, the launcher, and notes from each prototype |
| [`spike/`](spike/) | the drafting spike: isolation of model calls, the corpus, the Rust contract and validator, its workers-rs build |
| [`spec/tla/`](spec/tla/README.md) | a TLA+ model of the job protocol for the planned celld version |
| `mockups/` | HTML mock-ups of the earlier desktop design |

## About Contributions

Please don't take this the wrong way, but I do not accept outside contributions for any of my
projects. I simply don't have the mental bandwidth to review anything, and it's my name on the
thing, so I'm responsible for any problems it causes; thus, the risk-reward is highly asymmetric
from my perspective. I'd also have to worry about other "stakeholders," which seems unwise for
tools I mostly make for myself for free. Feel free to submit issues, and even PRs if you want to
illustrate a proposed fix, but know I won't merge them directly. Instead, I'll have Claude or
Codex review submissions via `gh` and independently decide whether and how to address them. Bug
reports in particular are welcome. Sorry if this offends, but I want to avoid wasted time and hurt
feelings. I understand this isn't in sync with the prevailing open-source ethos that seeks
community contributions, but it's the only way I can move at this velocity and keep my sanity.

## License

None yet: all rights reserved until a license is added.
