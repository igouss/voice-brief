# voice-brief — idea

Develop an idea by talking, with AI help, until it is a task prompt an agent can act on.

## The problem

My ideas for work arrive as speech, not prose. Today I dictate a rough paragraph into an agent
session and the agent rewrites it. That works, but:

- the raw words are lost once the agent has rewritten them, so the agent's paraphrase becomes the
  only record of what I meant;
- the agent fills the gaps in my idea with plausible guesses, and nothing marks which parts were
  mine;
- an idea needs several passes (say it, see it, correct it, add to it), and nothing holds it
  between passes or between sessions.

## What it does

A small Omarchy desktop app, bound to a key. I talk; my words are recorded and transcribed
locally; AI helps me shape them into a task prompt in a fixed format; I revise by voice and
keyboard until the prompt says what I mean; then I hand it to an agent in the project I am
working on (Herdr).

## The format

A task prompt states the change as a transformation, not as a plan:

| Field | What it says |
|---|---|
| **Weakest part** | One place in the system as it stands, specific enough that someone else could open it and check: a module, a crate edge, a file pair that always changes together. Found from evidence: files that churn or change together, bugs that cluster, a part you can't touch without understanding the whole (cm-igd). Not "the architecture", not "we lack a port layer". |
| **Force** | Two pressures the current structure can't both satisfy, at least one of them already observed: a bug, a build artifact, a change that hurt. A guess about the future is not a force (cm-xrl); a symptom ("this file is too big") is not either, the force is what it braids. |
| **Smallest move** | The least change that resolves that force and leaves everything else working. It follows from the force: if another pattern could be swapped in and the text would still read fine, it is not a diagnosis. It fits in one commit (cm-uom). |
| **Preserves** | What the move must leave intact |
| **Would reverse it** | The observable result that would show the move was wrong and should be undone |

Not a diagnosis: "apply pattern X" (a move looking for a force), a list of everything wrong (only
the one weakest part), a picture of the end state (a master plan). "Nothing is weak; no structural
change" is a valid result, and often the right one.

The fields come from two of my rules: weakest part, force and smallest move from cm-dr0 ("the
weakest part, the unresolved force, the smallest move"); preserves and would reverse it from
cm-e2y ("weakest part at the time, forces in conflict, move taken, what it preserved, what would
reverse it").

These definitions are the single source: the grade, the AI's questions and the spike all use
them (cm-eds).

Each field is filled from my own words. Where I have not said something, the AI asks; it does not
fill the field with a guess. "Would reverse it" is the field most likely to be missing and the one
a guess would harm most, because it is what makes the task falsifiable (cm-gi3).

## Where the format comes from

My reading of its lineage, not a note I wrote down (a search of `~/kbe` found no source):

- **Force:** Christopher Alexander, *Notes on the Synthesis of Form* (1964). Design resolves misfits
  between a form and its context; forces are the pressures in conflict. The pattern-language format
  (context, problem, forces, solution) carried the word into software patterns.
- **Weakest part, and a change that leaves the rest intact:** Alexander, *The Nature of Order*
  (2002–2005): structure grows by structure-preserving transformations, each strengthening what is
  weakest in the current whole. The concept is his; "weakest part" may not be his exact wording.
- **Decision records:** Michael Nygard, "Documenting Architecture Decisions" (2011): context,
  decision, status, consequences. cm-e2y differs by recording a transformation instead of a choice.
- **Would reverse it:** no single source; close to revisit triggers in some ADR templates and to
  one-way versus two-way doors. It makes the task falsifiable.

## Why the verbatim transcript matters

My words, with the date, are the evidence for what I asked (cm-6nu). A recording plus its
transcript lets every field of the prompt be checked against what I actually said, instead of the
agent's paraphrase replacing it.

## Constraints

- **CPU-only.** i7-10700K, no usable GPU compute. Speech-to-text on CPU is fine for short clips.
- **Local generation is too slow for the interactive path** (`voice-command-bar` measured about
  3.7 tok/s). AI work runs on Claude, through celld and pi-durable.
- **Omarchy / Hyprland.** Launch from a keybinding, float as a small window, follow the theme.

## Prior work to reuse, not rebuild

- `voxtype`: push-to-talk capture and transcription, with `record` for external control.
- `~/Projects/mine/voice-command-bar`: findings on local latency.
- `~/Projects/mine/voice-llm`: streaming TTS, which could read a prompt back to me.
- `pi-world`: celld and pi-durable in practice, including Claude through the subscription.

The design is in [`design.md`](design.md).
