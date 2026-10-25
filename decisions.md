# voice-brief — decisions

The operator's own words, the authority for the design in [design.md](design.md). Nothing
here is paraphrased.

Operator words, 2026-10-25:

- "I want to use the model when I use the app … I want the model to remain in the memory for a
  while … say for 10 min."
- "Append (at the current cursor position), revise - take a new recording and current version and
  showed a poposed revision, and a way to accept it. I should be able to edit it in the app, no
  need for the external editor."
- "History yes, in git." "I do want to be able to delete some transcripts and voice recording
  (they stay in git)."
- "since this would be a long running process, let's use pi-durable and celld I have."
- "I think one central repo." "Showing briefs in Herdr's sidebar - that would be cook, yeah!"
- "background process is not probelm."
- "I should be able to start a new agent for the project in herder with the current revision of
  the prompt, and I should be able to paste to the currently selected agent."
- "OK to make changes, those are my personal set-up projects."
- "this is a dictation app, to develop my idea using my microphone and AI in the format that I
  want: Weakest part, Force, Preserves, Would reverse it, format."
- "building a task prompt for the agent is the idea."
- "Weakest part, force, smallest move come from cm-dr0 … Preserves, would reverse it come from
  cm-e2y … Smallest move stays."
- "OK for one thread.jsonl per idea." "Regarding deletion, sometimes the idea needs a rephrasing,
  or to be deleted because transcipt is wrong."
- "one machine only, install tools locally" "ok model it now"
- "Yeah, transcription is weak, I would need to view and approve it before it enters into the
  system."
- "I think this workflow actually might work well. We don't need the dictation up from the vox we
  can install, not install, just use the dictation that the Claude Code provides. It seems to be
  pretty accurate and fast."
- Dictated as corpus take t3: "The application itself should or still support recording and
  transcribing locally through the vox when I'm not running it from the claude code instance."
  Then, asked whether to record it: "yes, add it to design.md".
- "to be a dual mode application. It should support Claude Code and it should support recording
  when it's running outside the Claude Code. For now, concentrate on the Claude Code version."
- "it seems stupid to have a rolling buffer when we could just intercept the web socket and
  capture the exact data that we capture."
- "TypeScript is fine but I don't like it as a language, WASM is fine too … Maybe we can do rust
  to wasm?" "use the standard workers-rs toolchain"
- "I don't want to send the dictated text to the model when I don't want to, and if I do want to
  send the dictated text, I will just press enter … For now, a preferred mode where I just open
  Claude and use its voice … for this application to populate voice notes."
- "Maybe we could just use Claude mods and this void capture and processing, and we just do it
  from the inside Claude code. Maybe we don't need to write an application right now. That would
  be a follow-up possibility. Let's concentrate right now on a prototype that uses Claude code
  mods and brings some kind of user interface for the application that we're building."
- "The voice proxy runs inside whichever pane first started it, so closing that pane can cut
  every proxied Claude session off at once. Fix that please. And let's update the documents to
  reflect this pivot. Let's organize our voice notes and everything else that we generate so that
  it's per Git project, so that we could safely run proxy the Claude code."
- "Okay, package this as a Claude Code plugin." Asked how to run the validator and set up the
  proxy: "Port to TypeScript", "/vb-setup command". Then: "the proper rewrite. Don't spawn using
  command line. Use a built-in claude code facilities to do this."
- "let's add a Rust + GPUI app that would integrate this app/plugin with my omarchy desktop,
  hide/show with F1, and design a nice looking minimalist user interface for it." Then: "which
  generate the model for the TypeScript from my Rust code. So we have a single source of truth."
- "I'm ok to set local settings for tap mode if it will simplify the integration." "you can have
  a hook that would stop the message from going to agent." After a test dictation reached the
  agent: "the message was sent to the agent. I don't want that."
