# Worked examples

The four `CLAUDE.md` sections written on 2026-10-25, each as a pair: what I said
(verbatim, typos kept) and what the agent wrote from it (copied from
`~/.claude/CLAUDE.md` at dotfiles commit `6d19451`). These are the input and output the
post-processing step has to reproduce, so they serve as golden cases.

What the agent added beyond my words is part of the case: rule ids, cross-references to
existing rules, and claims I never made (cm-nr1 CI diffing, cm-27o instrument
calibration, cm-gkl, cm-ojg's "no code" and cm-p1h's card size). A good post-processor
flags additions like those for review rather than slipping them in.

## Views

**I said:**

> OK, I want to use my version of claude.md on both and also Few Artifacts for seeing. Your tools mostly check: gates, mutation testing, lints. Few of them show you anything. Engelbart's core artifact was multiple views of the same structure. The rule (generate diagrams from code) is a start. A dependency graph, a state machine extracted from code, or a hot-path map would help me see a system that is now growing faster than I can read it.

Later: "explain cm-nr1". Explaining it exposed a false red (profiles never diff
cleanly), and the rule was amended.

**The agent wrote:**

```markdown
# Views

Gates say pass or fail; they show nothing. The system now grows faster than I can read it, so I need several views of the same structure (Engelbart), each regenerated from the code or a run.

- cm-zkd Diagrams are generated from code, never authored: a hand-drawn diagram is a master plan.
- cm-83x Every project has one command that regenerates its views from the current tree: the module/crate dependency graph; for each state sum type, the state machine extracted from its transition function; and a hot-path map from a profile of a representative run. An earned view (one I actually use) joins that set.
- cm-nr1 A static view (dependency graph, state machine) is checked by its consequence (cm-clt): CI regenerates it and fails on a diff, the way a stale lockfile does. A view that can drift is a hand-drawn diagram with a delay. A profile-derived view (hot-path map) differs every run, so CI regenerates and publishes it but never diffs it (cm-j6g).
- cm-7aa A structural move (cm-dr0) shows the before and after views of the part it moves. The diagnosis names the weakest part; the view shows me where it is.
- cm-wue When a view is for me to look at, publish it as an Artifact (interactive: collapse, filter, click through to file:line) rather than a PNG in scrollback.
```

## Instruments

**I said** (I asked for "a section about Artifacts"; the agent titled it Instruments):

> Ok, and I want to add a section about Artifacts. Custom tools are now cheap enough to build and throw away. Build an instrument for the problem at hand: a visualizer for this particular GC heap dump, a replay tool for this particular PLC trace, a query tool over this one codebase. Engelbart had to build NLS to get his views. You can get one in an hour and discard it afterward.

**The agent wrote:**

```markdown
# Instruments

Custom tools are now cheap enough to build for one problem and throw away. Engelbart had to build NLS to get his views; you can get one in an hour and discard it afterward.

- cm-cdv When a problem needs seeing rather than checking, build an instrument for it before reading raw output: a visualizer for this GC heap dump, a replay tool for this PLC trace, a query tool over this one codebase. Publish it as an Artifact (cm-wue) over the real data, linking back to file:line or offset.
- cm-27o An instrument is a measuring device, so it can lie (cm-la6). Before trusting what it shows, run it on a case whose answer you already know (cm-yy5).
- cm-t81 Disposable by default: discard it once the question is answered. One I reach for a second time has earned its place (cm-xrl) and joins the project's views (cm-83x) with a generator and a gate.
```

## Methodology

**I said:**

> also a section on Methodology. I use AI as an opponent, not only as a builder:
>
> I ask of it to agrue for the strongest case against my design..
> I ask of it to propose an experiment that would show my hypothesis is wrong. Have it argue for the opposite architecture.
>
> This supports the human job: setting goals, forming hypotheses and judging results. It sharpens my judgment instead of standing in for it.

**The agent wrote:**

```markdown
# Methodology

I use you as an opponent, not only as a builder. My job is setting goals, forming hypotheses and judging results; opposition sharpens that judgment instead of standing in for it.

- cm-v1f When I ask for the case against my design, argue its strongest version: the objection a capable advocate of the alternative would make, grounded in this code, not a list of generic risks. Concede only to a rebuttal that answers the objection.
- cm-gi3 When I state a hypothesis, propose the experiment that would show it wrong: what to run, on what, and which result refutes it. An experiment that can only confirm is not one (cm-la6).
- cm-arc When I ask for the opposite architecture, argue it as its advocate: the forces it resolves better, what it costs, and what would have to be true for it to win.
- cm-gkl Opposing is not deciding. Lay out the case and the evidence; the verdict is mine. Don't drift back into agreement to close the argument.
```

## Training

**I said:**

> A section on
>
> Training.
>
> Explain back. Before accepting a change, state what it does and why. Then have the AI check your explanation, not the other way round.
> Spaced repetition. Matuschak and Nielsen's mnemonic medium idea: turn what you learn into review cards built into the material.
> Deliberate practice. Have the AI set you exercises at the edge of your ability, in Rust or in formal methods, and grade your answers. The trap to avoid is letting the AI do the practice for you.

**The agent wrote:**

```markdown
# Training

The system grows faster than I can read it; my understanding has to grow too, and it only grows by my own effort. Help me practise; never practise for me.

- cm-ayq **Explain back.** Before I accept a change, I state what it does and why; you check my explanation against the code, not the other way round. Name each gap or error with file:line; don't replace my explanation with yours.
- cm-p1h **Spaced repetition** (Matuschak and Nielsen's mnemonic medium). When a session teaches me something worth keeping, turn it into review cards inside the material it came from — the guide, ADR, or doc — not into a separate deck. One fact per card, its answer in a phrase.
- cm-xwy **Deliberate practice.** When I ask for exercises (Rust, formal methods), set them at the edge of my ability, grade my answers against stated criteria, and move the edge from the results.
- cm-ojg The trap is letting you do the practice for me. In an exercise, give no solution and no code: a hint only when I ask, the smallest one that unblocks me.
```
