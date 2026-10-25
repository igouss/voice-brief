export default {
 "transcript": "The weakest part is the sync module. Every time I change the HTTP client,   the desktop build breaks. I don’t want a new port layer.",
 "cases": [
  {
   "name": "accepts zero claims when every field asks",
   "reply": "{\"fields\": {\"weakest_part\": {\"claims\": [], \"question\": \"What would show the move was wrong?\"}, \"force\": {\"claims\": [], \"question\": \"What would show the move was wrong?\"}, \"smallest_move\": {\"claims\": [], \"question\": \"What would show the move was wrong?\"}, \"preserves\": {\"claims\": [], \"question\": \"What would show the move was wrong?\"}, \"would_reverse_it\": {\"claims\": [], \"question\": \"What would show the move was wrong?\"}}}",
   "expect": "ok"
  },
  {
   "name": "accepts one claim with one span",
   "reply": "{\"fields\": {\"weakest_part\": {\"claims\": [{\"id\": \"c1\", \"text\": \"t\", \"said\": [\"the sync module\"]}]}, \"force\": {\"claims\": [], \"question\": \"What would show the move was wrong?\"}, \"smallest_move\": {\"claims\": [], \"question\": \"What would show the move was wrong?\"}, \"preserves\": {\"claims\": [], \"question\": \"What would show the move was wrong?\"}, \"would_reverse_it\": {\"claims\": [], \"question\": \"What would show the move was wrong?\"}}}",
   "expect": "ok"
  },
  {
   "name": "accepts many claims with many spans",
   "reply": "{\"fields\": {\"weakest_part\": {\"claims\": [], \"question\": \"What would show the move was wrong?\"}, \"force\": {\"claims\": [{\"id\": \"c1\", \"text\": \"t\", \"said\": [\"Every time I change the HTTP client\", \"the desktop build breaks\"]}, {\"id\": \"c2\", \"text\": \"t\", \"said\": [\"the sync module\"]}]}, \"smallest_move\": {\"claims\": [], \"question\": \"What would show the move was wrong?\"}, \"preserves\": {\"claims\": [], \"question\": \"What would show the move was wrong?\"}, \"would_reverse_it\": {\"claims\": [], \"question\": \"What would show the move was wrong?\"}}}",
   "expect": "ok"
  },
  {
   "name": "accepts an added claim with a reason",
   "reply": "{\"fields\": {\"weakest_part\": {\"claims\": [], \"question\": \"What would show the move was wrong?\"}, \"force\": {\"claims\": [], \"question\": \"What would show the move was wrong?\"}, \"smallest_move\": {\"claims\": [], \"question\": \"What would show the move was wrong?\"}, \"preserves\": {\"claims\": [{\"id\": \"c1\", \"text\": \"t\", \"added\": \"inferred from the build\"}]}, \"would_reverse_it\": {\"claims\": [], \"question\": \"What would show the move was wrong?\"}}}",
   "expect": "ok"
  },
  {
   "name": "accepts a straight apostrophe for a curly one",
   "reply": "{\"fields\": {\"weakest_part\": {\"claims\": [], \"question\": \"What would show the move was wrong?\"}, \"force\": {\"claims\": [{\"id\": \"c1\", \"text\": \"t\", \"said\": [\"I don't want a new port layer\"]}]}, \"smallest_move\": {\"claims\": [], \"question\": \"What would show the move was wrong?\"}, \"preserves\": {\"claims\": [], \"question\": \"What would show the move was wrong?\"}, \"would_reverse_it\": {\"claims\": [], \"question\": \"What would show the move was wrong?\"}}}",
   "expect": "ok"
  },
  {
   "name": "accepts a span whose whitespace differs",
   "reply": "{\"fields\": {\"weakest_part\": {\"claims\": [], \"question\": \"What would show the move was wrong?\"}, \"force\": {\"claims\": [{\"id\": \"c1\", \"text\": \"t\", \"said\": [\"HTTP client, the desktop\\nbuild\"]}]}, \"smallest_move\": {\"claims\": [], \"question\": \"What would show the move was wrong?\"}, \"preserves\": {\"claims\": [], \"question\": \"What would show the move was wrong?\"}, \"would_reverse_it\": {\"claims\": [], \"question\": \"What would show the move was wrong?\"}}}",
   "expect": "ok"
  },
  {
   "name": "rejects a code fence",
   "reply": "```json\n{}\n```",
   "expect": [
    "malformed"
   ]
  },
  {
   "name": "rejects an unknown field",
   "reply": "{\"fields\": {\"weakest_part\": {\"claims\": [], \"question\": \"What would show the move was wrong?\"}, \"force\": {\"claims\": [], \"question\": \"What would show the move was wrong?\"}, \"smallest_move\": {\"claims\": [], \"question\": \"What would show the move was wrong?\"}, \"preserves\": {\"claims\": [], \"question\": \"What would show the move was wrong?\"}, \"would_reverse_it\": {\"claims\": [], \"question\": \"What would show the move was wrong?\"}, \"extra\": {\"claims\": [], \"question\": \"What would show the move was wrong?\"}}}",
   "expect": [
    "unknown_field"
   ]
  },
  {
   "name": "rejects a missing field",
   "reply": "{\"fields\": {\"weakest_part\": {\"claims\": [], \"question\": \"What would show the move was wrong?\"}, \"force\": {\"claims\": [], \"question\": \"What would show the move was wrong?\"}, \"smallest_move\": {\"claims\": [], \"question\": \"What would show the move was wrong?\"}, \"would_reverse_it\": {\"claims\": [], \"question\": \"What would show the move was wrong?\"}}}",
   "expect": [
    "missing_field"
   ]
  },
  {
   "name": "rejects an empty field without a question",
   "reply": "{\"fields\": {\"weakest_part\": {\"claims\": [], \"question\": \"What would show the move was wrong?\"}, \"force\": {\"claims\": []}, \"smallest_move\": {\"claims\": [], \"question\": \"What would show the move was wrong?\"}, \"preserves\": {\"claims\": [], \"question\": \"What would show the move was wrong?\"}, \"would_reverse_it\": {\"claims\": [], \"question\": \"What would show the move was wrong?\"}}}",
   "expect": [
    "empty_without_question"
   ]
  },
  {
   "name": "rejects a blank question",
   "reply": "{\"fields\": {\"weakest_part\": {\"claims\": [], \"question\": \"What would show the move was wrong?\"}, \"force\": {\"claims\": [], \"question\": \"  \"}, \"smallest_move\": {\"claims\": [], \"question\": \"What would show the move was wrong?\"}, \"preserves\": {\"claims\": [], \"question\": \"What would show the move was wrong?\"}, \"would_reverse_it\": {\"claims\": [], \"question\": \"What would show the move was wrong?\"}}}",
   "expect": [
    "empty_without_question"
   ]
  },
  {
   "name": "rejects claims together with a question",
   "reply": "{\"fields\": {\"weakest_part\": {\"claims\": [], \"question\": \"What would show the move was wrong?\"}, \"force\": {\"claims\": [{\"id\": \"c1\", \"text\": \"t\", \"said\": [\"the sync module\"]}], \"question\": \"q\"}, \"smallest_move\": {\"claims\": [], \"question\": \"What would show the move was wrong?\"}, \"preserves\": {\"claims\": [], \"question\": \"What would show the move was wrong?\"}, \"would_reverse_it\": {\"claims\": [], \"question\": \"What would show the move was wrong?\"}}}",
   "expect": [
    "claims_and_question"
   ]
  },
  {
   "name": "rejects a claim without a source",
   "reply": "{\"fields\": {\"weakest_part\": {\"claims\": [], \"question\": \"What would show the move was wrong?\"}, \"force\": {\"claims\": [{\"id\": \"c1\", \"text\": \"t\"}]}, \"smallest_move\": {\"claims\": [], \"question\": \"What would show the move was wrong?\"}, \"preserves\": {\"claims\": [], \"question\": \"What would show the move was wrong?\"}, \"would_reverse_it\": {\"claims\": [], \"question\": \"What would show the move was wrong?\"}}}",
   "expect": [
    "claim_without_source"
   ]
  },
  {
   "name": "rejects a claim with both sources",
   "reply": "{\"fields\": {\"weakest_part\": {\"claims\": [], \"question\": \"What would show the move was wrong?\"}, \"force\": {\"claims\": [{\"id\": \"c1\", \"text\": \"t\", \"said\": [\"the sync module\"], \"added\": \"r\"}]}, \"smallest_move\": {\"claims\": [], \"question\": \"What would show the move was wrong?\"}, \"preserves\": {\"claims\": [], \"question\": \"What would show the move was wrong?\"}, \"would_reverse_it\": {\"claims\": [], \"question\": \"What would show the move was wrong?\"}}}",
   "expect": [
    "claim_with_both_sources"
   ]
  },
  {
   "name": "rejects a claim id used twice in one field",
   "reply": "{\"fields\": {\"weakest_part\": {\"claims\": [], \"question\": \"What would show the move was wrong?\"}, \"force\": {\"claims\": [{\"id\": \"c1\", \"text\": \"t\", \"said\": [\"the sync module\"]}, {\"id\": \"c1\", \"text\": \"t\", \"said\": [\"the desktop build breaks\"]}]}, \"smallest_move\": {\"claims\": [], \"question\": \"What would show the move was wrong?\"}, \"preserves\": {\"claims\": [], \"question\": \"What would show the move was wrong?\"}, \"would_reverse_it\": {\"claims\": [], \"question\": \"What would show the move was wrong?\"}}}",
   "expect": [
    "duplicate_claim_id"
   ]
  },
  {
   "name": "rejects a claim id used in two fields",
   "reply": "{\"fields\": {\"weakest_part\": {\"claims\": [], \"question\": \"What would show the move was wrong?\"}, \"force\": {\"claims\": [{\"id\": \"c1\", \"text\": \"t\", \"said\": [\"the sync module\"]}]}, \"smallest_move\": {\"claims\": [], \"question\": \"What would show the move was wrong?\"}, \"preserves\": {\"claims\": [{\"id\": \"c1\", \"text\": \"t\", \"said\": [\"the desktop build breaks\"]}]}, \"would_reverse_it\": {\"claims\": [], \"question\": \"What would show the move was wrong?\"}}}",
   "expect": [
    "duplicate_claim_id"
   ]
  },
  {
   "name": "rejects a span one word off the transcript",
   "reply": "{\"fields\": {\"weakest_part\": {\"claims\": [{\"id\": \"c1\", \"text\": \"t\", \"said\": [\"the sync layer\"]}]}, \"force\": {\"claims\": [], \"question\": \"What would show the move was wrong?\"}, \"smallest_move\": {\"claims\": [], \"question\": \"What would show the move was wrong?\"}, \"preserves\": {\"claims\": [], \"question\": \"What would show the move was wrong?\"}, \"would_reverse_it\": {\"claims\": [], \"question\": \"What would show the move was wrong?\"}}}",
   "expect": [
    "span_not_in_transcript"
   ]
  },
  {
   "name": "rejects an empty span",
   "reply": "{\"fields\": {\"weakest_part\": {\"claims\": [], \"question\": \"What would show the move was wrong?\"}, \"force\": {\"claims\": [{\"id\": \"c1\", \"text\": \"t\", \"said\": [\" \"]}]}, \"smallest_move\": {\"claims\": [], \"question\": \"What would show the move was wrong?\"}, \"preserves\": {\"claims\": [], \"question\": \"What would show the move was wrong?\"}, \"would_reverse_it\": {\"claims\": [], \"question\": \"What would show the move was wrong?\"}}}",
   "expect": [
    "empty_span"
   ]
  },
  {
   "name": "rejects a span that differs only in case",
   "reply": "{\"fields\": {\"weakest_part\": {\"claims\": [], \"question\": \"What would show the move was wrong?\"}, \"force\": {\"claims\": [{\"id\": \"c1\", \"text\": \"t\", \"said\": [\"The Sync Module\"]}]}, \"smallest_move\": {\"claims\": [], \"question\": \"What would show the move was wrong?\"}, \"preserves\": {\"claims\": [], \"question\": \"What would show the move was wrong?\"}, \"would_reverse_it\": {\"claims\": [], \"question\": \"What would show the move was wrong?\"}}}",
   "expect": [
    "span_not_in_transcript"
   ]
  }
 ]
}
