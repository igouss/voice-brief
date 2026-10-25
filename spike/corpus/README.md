# Corpus

Four ideas Iouri dictated on 2026-10-25 with `proto/vb_proto.py`'s `r` key (voxtype). Each
transcript was edited and then approved before it entered the thread. `corpus.json` holds, for
each idea:

- the approved transcript, and what voxtype heard;
- the claims: verbatim spans, each with its field (one of the five, or `none` for context);
- the fields left empty;
- the operator's confirmation, quoted.

The annotation was drafted by the agent and confirmed by the operator. Four judgement calls were
put to the operator and accepted as drafted:

- t3's "live with that" counts as a Smallest move;
- t3's local voxtype recording counts as Preserves;
- t6's "easily accessible with a short key" counts as Force;
- t4 has no Weakest part.

`check_spans.py corpus.json` fails if a span is not verbatim in its transcript, if a field name is
unknown, or if no idea leaves Would reverse it or Preserves empty. It was seen to fail on a span
with text appended.

The WAVs are in `audio/`, which is gitignored by operator decision: "let's not track it in git
yet".
