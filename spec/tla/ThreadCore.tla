---- MODULE ThreadCore ----
EXTENDS Naturals, Sequences, FiniteSets

CONSTANTS Jobs, Takes, MaxBase,
          DurableSubmit, Dedupe, GuardAccept, AppendOnDelete

None == "-"

EventTypes == {"Typed", "TakeRecorded", "TakeDeleted", "JobSubmitted",
               "ProposalMade", "JobFailed", "Accepted", "Rejected"}

InputKinds == {"Type", "Record", "Delete", "Start", "Accept", "Reject", "Result", "Recover"}

EffectKinds == {"Submit", "Ack"}

Events == [type : EventTypes, job : Jobs \cup {None}, take : Takes \cup {None},
           base : 0..MaxBase, ctx : SUBSET Takes]

JobSpecs == [job : Jobs, base : 0..MaxBase, ctx : SUBSET Takes]

Effects == [kind : EffectKinds, job : Jobs, base : 0..MaxBase, ctx : SUBSET Takes]

Inputs == [kind : InputKinds, job : Jobs \cup {None}, take : Takes \cup {None},
           outcome : {"ok", "fail", None}, base : 0..MaxBase, ctx : SUBSET Takes]

Ev(ty, j, t, b, c) == [type |-> ty, job |-> j, take |-> t, base |-> b, ctx |-> c]
Fx(k, j, b, c) == [kind |-> k, job |-> j, base |-> b, ctx |-> c]
In(k, j, t, o, b, c) == [kind |-> k, job |-> j, take |-> t, outcome |-> o, base |-> b, ctx |-> c]

Idx(s, ty) == {i \in DOMAIN s : s[i].type \in ty}
JobsOf(s, ty) == {s[i].job : i \in Idx(s, ty)}
TakesOf(s, ty) == {s[i].take : i \in Idx(s, ty)}

Rev(s) == Cardinality(Idx(s, {"Typed", "Accepted"}))
Live(s) == TakesOf(s, {"TakeRecorded"}) \ TakesOf(s, {"TakeDeleted"})
Results(s) == JobsOf(s, {"ProposalMade", "JobFailed"})
Proposed(s) == JobsOf(s, {"ProposalMade"})
Decided(s) == JobsOf(s, {"Accepted", "Rejected"})
ProposalOf(s, j) == s[CHOOSE i \in Idx(s, {"ProposalMade"}) : s[i].job = j]

Fresh(s, j) ==
  /\ ProposalOf(s, j).base = Rev(s)
  /\ ProposalOf(s, j).ctx \subseteq Live(s)

PendingSpecs(st) ==
  IF DurableSubmit
  THEN {[job |-> st.thread[i].job, base |-> st.thread[i].base, ctx |-> st.thread[i].ctx] :
          i \in {k \in Idx(st.thread, {"JobSubmitted"}) : st.thread[k].job \notin Results(st.thread)}}
  ELSE {p \in st.pending : p.job \notin Results(st.thread)}

Started(st) == JobsOf(st.thread, {"JobSubmitted"}) \cup {p.job : p \in st.pending} \cup Results(st.thread)

Empty == [thread |-> <<>>, pending |-> {}]

Out(st, fx) == [st |-> st, fx |-> fx]
Push(st, e) == [st EXCEPT !.thread = Append(@, e)]

RemoveRecorded(s, t) == SelectSeq(s, LAMBDA e : ~(e.type = "TakeRecorded" /\ e.take = t))

StepType(st) == Out(Push(st, Ev("Typed", None, None, 0, {})), {})

StepRecord(st, t) ==
  IF t \in TakesOf(st.thread, {"TakeRecorded"})
  THEN Out(st, {})
  ELSE Out(Push(st, Ev("TakeRecorded", None, t, 0, {})), {})

StepDelete(st, t) ==
  IF t \notin Live(st.thread) THEN Out(st, {})
  ELSE IF AppendOnDelete THEN Out(Push(st, Ev("TakeDeleted", None, t, 0, {})), {})
  ELSE Out([st EXCEPT !.thread = RemoveRecorded(@, t)], {})

StepStart(st, j) ==
  IF j \in Started(st) THEN Out(st, {})
  ELSE LET b == Rev(st.thread)
           c == Live(st.thread)
       IN IF DurableSubmit
          THEN Out(Push(st, Ev("JobSubmitted", j, None, b, c)), {Fx("Submit", j, b, c)})
          ELSE Out([st EXCEPT !.pending = @ \cup {[job |-> j, base |-> b, ctx |-> c]}],
                   {Fx("Submit", j, b, c)})

StepResult(st, in) ==
  IF Dedupe /\ in.job \in Results(st.thread)
  THEN Out(st, {Fx("Ack", in.job, 0, {})})
  ELSE Out(Push(st, IF in.outcome = "ok"
                    THEN Ev("ProposalMade", in.job, None, in.base, in.ctx)
                    ELSE Ev("JobFailed", in.job, None, 0, {})),
           {Fx("Ack", in.job, 0, {})})

StepDecide(st, j, ty) ==
  IF /\ j \in Proposed(st.thread) \ Decided(st.thread)
     /\ (ty = "Accepted" /\ GuardAccept) => Fresh(st.thread, j)
  THEN Out(Push(st, Ev(ty, j, None, 0, {})), {})
  ELSE Out(st, {})

StepRecover(st) == Out(st, {Fx("Submit", p.job, p.base, p.ctx) : p \in PendingSpecs(st)})

Step(st, in) ==
  CASE in.kind = "Type"    -> StepType(st)
    [] in.kind = "Record"  -> StepRecord(st, in.take)
    [] in.kind = "Delete"  -> StepDelete(st, in.take)
    [] in.kind = "Start"   -> StepStart(st, in.job)
    [] in.kind = "Accept"  -> StepDecide(st, in.job, "Accepted")
    [] in.kind = "Reject"  -> StepDecide(st, in.job, "Rejected")
    [] in.kind = "Result"  -> StepResult(st, in)
    [] in.kind = "Recover" -> StepRecover(st)

====
