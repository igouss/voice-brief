# Job protocol model

A TLA+ model of how `vbd`, the celld jobs and an idea's thread interleave. Run it with
`spec/tla/check.sh` (TLC from `~/.local/opt/tlaplus`, Java from mise); it passes only if the
design passes and every red variant fails for its stated reason.

## Shape

- **`ThreadCore.tla`** is `vbd`'s pure transition, `Step(state, input) -> [st, fx]`: the same
  function the Rust domain will have (cm-prk), and later a Kani harness target. Guards live inside
  `Step`, so an input that is not allowed returns the state unchanged with no effects. Effects are
  data: `Submit` and `Ack`.
- **`JobThread.tla`** adds only the environment the code does not contain: the actors, their
  interleaving, and each external dependency as a contract.

## Actors and interleaving

| Actor | Steps | Contract assumed |
|---|---|---|
| Operator | any input: type, record or delete a take, start, accept or reject a job | none; no fairness |
| `vbd` | applies `Step` to each input; on restart applies `Recover` | the thread is durable on append; memory (pending jobs, outbox) is lost on a crash |
| celld | takes `Submit` from the outbox, runs the job, delivers its result, takes `Ack` | a job id is created once: a second `Submit` for it is ignored; a running job ends `ok` or `fail`; a finished, unacknowledged result is delivered at least once, possibly many times; `Ack` stops delivery |
| Host | crashes `vbd` at any time (bounded), restarts it | restart eventually happens |

Steps interleave freely through `Next`. Fairness covers only the environment (celld steps and
restart), never the operator.

## State variables that decide correctness

`core.thread` (the record), `core.pending` (`vbd`'s in-memory jobs, used only by the volatile
variant), `outbox` (effects not yet taken by celld), `celld` (each job's status and the base
revision and transcripts it was computed from), `acked`, `vbdUp`. `requested` is a ghost variable
for the liveness property; `crashes` bounds the model.

## Properties

| Property | Kind | Says |
|---|---|---|
| `AppendOnly` | action | every step's thread starts with the previous thread |
| `AtMostOneResult` | invariant | each job has at most one result event in the thread |
| `AcceptOnlyFresh` | invariant | an accepted proposal was computed against the current revision, and every transcript it used is still live |
| `EveryJobEnds` | liveness | every started job eventually has a result or failure event in the thread |

## What the design needs, shown by the red variants

Each rule below is a constant; turning it off makes TLC find the bug it prevents.

| Rule | Variant | Counterexample TLC finds |
|---|---|---|
| `JobSubmitted` is appended to the thread before `Submit` is sent (`DurableSubmit`) | `RedVolatileSubmit` | start a job, crash before celld takes the submit, restart: the job never ends |
| a result for a job that already has one is acknowledged, not appended (`Dedupe`) | `RedNoDedupe` | celld delivers the same result twice before the ack: two results |
| accept only a fresh proposal (`GuardAccept`) | `RedNoGuard` | start a job, type, the result arrives, accept: a stale proposal lands |
| deletion appends `TakeDeleted` (`AppendOnDelete`) | `RedRewriteOnDelete` | deleting a take removes its `TakeRecorded`: history rewritten |

## Results, 2026-10-25

TLC 2.19, 4 workers, bounds: 2 jobs, 1 take, 1 typed revision, 1 crash.

- `Safety`: no error; 948,887 distinct states, depth 18, about 20 s.
- `Liveness`: no error; same state space, about 80 s.
- Each red variant fails with the message `check.sh` expects; the script itself was seen to fail
  when an expectation was wrong.

## Limits

- Bounded: more jobs, takes, revisions or crashes were not checked.
- One field: any revision makes every open proposal stale. The app tracks revisions per field;
  a per-field model would let proposals for other fields stay fresh.
- One host: `vbd` is the only writer of `~/briefs` (operator decision, 2026-10-25). Two writers
  would need a merge rule and a different model.
- The transport between `vbd` and celld is assumed to retry while `vbd` is up; only a crash loses
  effects.
- Not modelled: the speech-to-text timer, Herdr, git commits after the append.
