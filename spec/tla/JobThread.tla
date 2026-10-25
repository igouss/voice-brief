---- MODULE JobThread ----
EXTENDS ThreadCore

CONSTANTS MaxRev, MaxCrashes

VARIABLES core, outbox, vbdUp, celld, acked, requested, crashes

vars == <<core, outbox, vbdUp, celld, acked, requested, crashes>>

env == <<celld, acked, crashes>>

Apply(in) ==
  LET r == Step(core, in)
  IN  /\ core' = r.st
      /\ outbox' = outbox \cup r.fx

Init ==
  /\ core = Empty
  /\ outbox = {}
  /\ vbdUp = TRUE
  /\ celld = [j \in Jobs |-> [st |-> "none", base |-> 0, ctx |-> {}]]
  /\ acked = {}
  /\ requested = {}
  /\ crashes = 0

OperatorInputs ==
  {In("Type", None, None, None, 0, {})}
  \cup {In(k, None, t, None, 0, {}) : k \in {"Record", "Delete"}, t \in Takes}
  \cup {In(k, j, None, None, 0, {}) : k \in {"Start", "Accept", "Reject"}, j \in Jobs}

Operator ==
  /\ vbdUp
  /\ \E in \in OperatorInputs :
       /\ in.kind = "Type" => Rev(core.thread) < MaxRev
       /\ Apply(in)
       /\ requested' = IF in.kind = "Start" THEN requested \cup {in.job} ELSE requested
  /\ UNCHANGED <<vbdUp, celld, acked, crashes>>

CelldSubmit(j) ==
  /\ \E e \in outbox :
       /\ e.kind = "Submit" /\ e.job = j
       /\ outbox' = outbox \ {e}
       /\ celld' = IF celld[j].st = "none"
                   THEN [celld EXCEPT ![j] = [st |-> "running", base |-> e.base, ctx |-> e.ctx]]
                   ELSE celld
  /\ UNCHANGED <<core, vbdUp, acked, requested, crashes>>

CelldAck(j) ==
  /\ \E e \in outbox :
       /\ e.kind = "Ack" /\ e.job = j
       /\ outbox' = outbox \ {e}
  /\ acked' = acked \cup {j}
  /\ UNCHANGED <<core, vbdUp, celld, requested, crashes>>

CelldRun(j) ==
  /\ celld[j].st = "running"
  /\ \E o \in {"ok", "fail"} : celld' = [celld EXCEPT ![j].st = o]
  /\ UNCHANGED <<core, outbox, vbdUp, acked, requested, crashes>>

CelldDeliver(j) ==
  /\ vbdUp
  /\ celld[j].st \in {"ok", "fail"}
  /\ j \notin acked
  /\ Apply(In("Result", j, None, celld[j].st, celld[j].base, celld[j].ctx))
  /\ UNCHANGED <<vbdUp, celld, acked, requested, crashes>>

Crash ==
  /\ vbdUp
  /\ crashes < MaxCrashes
  /\ vbdUp' = FALSE
  /\ core' = [core EXCEPT !.pending = {}]
  /\ outbox' = {}
  /\ crashes' = crashes + 1
  /\ UNCHANGED <<celld, acked, requested>>

Restart ==
  /\ ~vbdUp
  /\ vbdUp' = TRUE
  /\ Apply(In("Recover", None, None, None, 0, {}))
  /\ UNCHANGED <<celld, acked, requested, crashes>>

Next ==
  \/ Operator
  \/ \E j \in Jobs : CelldSubmit(j) \/ CelldAck(j) \/ CelldRun(j) \/ CelldDeliver(j)
  \/ Crash
  \/ Restart

Fairness ==
  /\ WF_vars(Restart)
  /\ \A j \in Jobs : /\ WF_vars(CelldSubmit(j)) /\ WF_vars(CelldAck(j))
                     /\ WF_vars(CelldRun(j)) /\ WF_vars(CelldDeliver(j))

Spec == Init /\ [][Next]_vars /\ Fairness

TypeOK ==
  /\ core.thread \in Seq(Events)
  /\ core.pending \subseteq JobSpecs
  /\ outbox \subseteq Effects
  /\ vbdUp \in BOOLEAN
  /\ celld \in [Jobs -> [st : {"none", "running", "ok", "fail"}, base : 0..MaxBase, ctx : SUBSET Takes]]
  /\ acked \subseteq Jobs
  /\ requested \subseteq Jobs
  /\ crashes \in 0..MaxCrashes

AtMostOneResult ==
  \A j \in Jobs :
    Cardinality({i \in Idx(core.thread, {"ProposalMade", "JobFailed"}) : core.thread[i].job = j}) <= 1

AcceptOnlyFresh ==
  \A i \in Idx(core.thread, {"Accepted"}) :
    LET before == SubSeq(core.thread, 1, i - 1)
    IN  Fresh(before, core.thread[i].job)

AppendOnly ==
  [][/\ Len(core.thread') >= Len(core.thread)
     /\ SubSeq(core.thread', 1, Len(core.thread)) = core.thread]_vars

EveryJobEnds == \A j \in Jobs : (j \in requested) ~> (j \in Results(core.thread))

====
