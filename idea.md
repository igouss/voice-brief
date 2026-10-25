# voice-brief — idea

Speak a definition; get back the rule I meant and a prompt an agent can act on.

## The problem

My working definitions (the sections of `~/.claude/CLAUDE.md`: Views, Instruments,
Methodology, Training, and the ones still to come) arrive as speech, not prose. Today I
dictate or paste a rough paragraph into a Claude session, and the agent turns it into
rules, mints ids, commits and syncs. That works, but:

- the raw words are lost once the agent has rewritten them, so the agent's paraphrase
  becomes the only record of what I said;
- every capture costs a full agent session, even for one sentence;
- nothing collects definitions between sessions, so ideas die between thinking one and
  opening a terminal.

## What it does

A small Omarchy desktop app, bound to a key, with one loop:

1. **Record.** Push-to-talk, the same gesture as `voxtype` dictation. Audio is kept.
2. **Transcribe.** Local speech-to-text (whisper, already on this machine through
   `voxtype` and `whisper-cli`). The verbatim transcript is the source of truth.
3. **Post-process.** Turn the transcript into one of two outputs, chosen at capture time
   or inferred:
   - **A definition**: a section or rule in the `CLAUDE.md` house style (bold lead,
     force stated, pointer to the gate), with a freshly minted `cm-` id that is checked
     against every id already in use.
   - **A task prompt**: a brief for an agent that states what, not how (cm-v6i), and
     carries my words verbatim with the date, ready for an epic's `## Operator decision`
     section (cm-6nu).
4. **Review.** Show the transcript and the draft side by side. I edit, accept or discard.
   Nothing is written anywhere without my accept.
5. **Hand off.** An accepted definition becomes a proposed edit to `CLAUDE.md` in the
   dotfiles repo (a commit I approve, then the existing post-commit push and SessionStart
   sync carry it to every host). An accepted task prompt is copied, saved to an inbox,
   or opens a Claude Code session with it.

## Why the verbatim transcript matters

cm-6nu wants my words verbatim with the date. A recording plus its transcript is the
strongest form of that evidence: the agent's paraphrase can be checked against what I
actually said, instead of replacing it.

## Constraints

- **CPU-only.** i7-10700K, no usable GPU compute (Tahiti hangs). Whisper on CPU is fine
  for short clips.
- **Local generation is too slow for the interactive path.** `voice-command-bar` measured
  about 3.7 tok/s locally; rewriting one paragraph would take minutes. Post-processing
  is generative, so it either runs remotely (`claude -p` or the API) or runs in the
  background with the review step waiting.
- **Omarchy / Hyprland.** Launch from a keybinding, float as a small window, follow the
  theme.

## Prior work to reuse, not rebuild

- `voxtype`: push-to-talk capture and whisper transcription, with a `record` command
  for external control and a `transcribe` command for files.
- `~/Projects/mine/voice-command-bar`: findings on local latency and on keeping the
  generative LLM off the interactive path.
- `~/Projects/mine/voice-llm` (claude-speech): streaming TTS for the Stop hook. It would
  let the app read a draft back to me.

## Open questions (mine to decide)

- Where does post-processing run: Claude remotely, the slow local model in the
  background, or both with a fallback?
- Which outputs land automatically, and which only on explicit accept? (The default
  above is: nothing without accept.)
- Should a definition go straight to a `CLAUDE.md` diff, or to a review inbox first?
- Should the audio be kept, for how long, and where (dotfiles is wrong: it's binary and
  personal)?
- Do task prompts target only Claude Code, or any agent (pi, Codex)?
- Later: does the explain-back drill (cm-ayq) fit here, with me speaking my explanation
  of a change and an agent checking it against the code?

## Golden cases

[`examples.md`](examples.md) holds the four sections written so far, my words next to
what the agent wrote. Post-processing should reproduce them and flag what it adds.
