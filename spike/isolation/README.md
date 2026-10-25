# Isolation: model calls see only the explicit prompt

Spike bead `voice-brief-drafting-spike-ong.2`. The production job runs on celld with an explicit
prompt and nothing else. A `claude -p` call loads `~/.claude/CLAUDE.md`, memory and skills, so a
spike run that way would measure a better-briefed system.

## Mechanism

`run.sh isolated <system-prompt-file> <prompt-file> <out-dir>` runs `claude -p`:

- from a new empty directory;
- with `CLAUDE_CODE_DISABLE_CLAUDE_MDS=1` and `CLAUDE_CODE_DISABLE_AUTO_MEMORY=1`;
- with `--system-prompt` (replaces Claude Code's system prompt), `--setting-sources ""` (no
  settings files or their hooks), `--tools ""`, `--strict-mcp-config`,
  `--disable-slash-commands`, `--no-session-persistence` and
  `--exclude-dynamic-system-prompt-sections`.

It uses the subscription login. `--bare` would be cleaner but needs a paid API key, which the
bead says to ask about first.

## How to confirm nothing else was loaded

`run.sh` sends the call through a local `mitmdump` on port 8898. `capture.py` saves each
`/v1/messages` request body under `<out-dir>/requests/`. These are the exact bytes sent, not what
the model chooses to say about its context.

`verify.py leaks|clean <out-dir>` looks in those bodies for text that only the user context
carries: a `CLAUDE.md` rule ("False green", "Diagnose before you design"), `MEMORY.md`,
`voice-brief` and `skills`.

The request bodies hold the account and device ids and the user's email, so `requests/` and
`proxy.log` are gitignored. The replies are kept.

## Runs, 2026-10-25, claude-opus-5-5, Claude Code 2.1.310

| Run | Canary reply | `verify.py` |
|---|---|---|
| `20261025T084953-default` | quoted cm-la6 and named `/home/elendal/.claude/CLAUDE.md` | `leaks`: False green 1, Diagnose before you design 1, MEMORY.md 2, voice-brief 3, skills 26 |
| `20261025T084953-isolated` | `NONE`, "No file contents were given to me." | not run (before the last flag) |
| `20261025T085040-isolated2` | `NONE`, "No file contents were given to me." | `clean`: none found |

Red first: `verify.py clean` on the default run fails, so the check can fail.

## What still reaches the model

These are not part of the explicit prompt, and Claude Code adds them anyway:

- a system line, "You are a Claude agent, built on Anthropic's Claude Agent SDK.", and a billing
  header line;
- a user-context reminder with the user's email address;
- an environment block: the empty temp directory, platform, OS version, model id, knowledge
  cutoff and date.

None carries a rule, a field definition or a project name. `--exclude-dynamic-system-prompt-sections`
did not remove the environment block. The celld job will have its own, different envelope; these
lines are the known difference between the spike and production.
