# PROTOTYPE — delete when answered

`vb_proto.py` is throwaway. Run: `python3 proto/vb_proto.py`.

## Question

Do my dictated words land in the five fields, quoted, with a question where I said nothing? And
does the loop of saying something, seeing it placed, and correcting it feel right?

## What it is

A stdlib Python REPL. It uses `pw-record` (16 kHz mono) and `voxtype transcribe` (parakeet,
loaded on each call). Then `claude -p` places the words into the five fields. The field
definitions are read from `idea.md` at runtime. The thread is kept in memory and written to the
temp directory when you quit.

Not in it: ratatui, `vbd`, celld, Herdr, git, model residency, or the re-check of claims that cite
a deleted take.

## Seen on the first run (synthetic speech, 2026-10-25)

- Every field I filled came back as `said t1` with exact spans. "Would reverse it" came back empty
  with a question.
- The question offered example answers ("for example, the desktop build still pulls in the HTTP
  client"). That is the AI suggesting content in the field it must not guess.
- voxtype heard "reqwest" as "request", and the AI carried it into Force. So correcting a
  transcript matters from the first take.
- Deleting the take after accepting left the fields built from it unchanged. The design's
  re-check is needed.
- Timing: transcription about 3.7 s, of which 2.4 s is loading the model; placement about 14 s.

## Seen on the first live run (Iouri's mic, 2026-10-25)

Takes: t1 a mic test, t2 "Yeah.", t3 an empty take (a mistyped `f` path recorded silently, now
refused), t4 a real idea about 50 s long.

- **The take names the fields as markers** ("Weakest part … Force … smallest move … preserves …
  would reverse it"). It's a natural way to dictate, and the placement can use it.
- **Speech-to-text loses the jargon.** "clawed" is probably `~/.claude`; "overwriting boosters
  work" is probably "overwriting loses work"; "would reserve it" is "would reverse it"; "DevBox is
  trapped" is unclear. Parakeet has no vocabulary for paths and project names.
- **The AI didn't guess "clawed"; it asked which file.** That keeps fidelity but costs a round,
  where correcting the transcript would have fixed it in one.
- **A suggestion inside the question happened twice out of two runs.** The Force question offered
  "for example, Fedora's copy has to stay immutable, but it also has to match the DevBox copy";
  the reversal question suggested its own answer.
- On the mic-test takes, it asked all five questions and filled nothing. Correct.
- Placement took 7 to 13 s; transcribing 50 s of audio took 5.5 s, model load included.
- The thread didn't record the proposal's field texts, so a run can't be reviewed afterwards.
  Fixed: `ProposalMade` and `TakeRecorded` now carry them.

## Verdict

- Transcript review comes first. Iouri, 2026-10-25: "Yeah, transcription is weak, I would need to
  view and approve it before it enters into the system." The prototype now shows each transcript
  and offers approve, edit or discard. Only an approved text is appended as `TakeRecorded`, which
  keeps both what voxtype heard and the approved text.
- (placement fidelity: still to judge on a live take)
