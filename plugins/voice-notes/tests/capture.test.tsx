import { expect, mock, test } from 'claude-code/testing'

const PANE = {
  plugin: 'voice-notes',
  component: 'Pane',
  requestId: 'voice-notes',
  props: { title: 'Voice notes', isFocused: false, bodyColumns: 80, placement: 'dock', scroll: { offset: 0, bodyRows: 40 }, view: {} },
} as const

const row = (pid: number, final: string): string =>
  JSON.stringify({ name: `s-${pid}`, start: 1001, pid, final, wav: '/w.wav', closed_by_client: true, endpoint: true })

async function start($: any, on: any, index: string, localSettings = '{}'): Promise<{ advance: (ms: number) => Promise<void> }> {
  mock.env(on, { HOME: '/h', HTTPS_PROXY: 'http://127.0.0.1:8897' })
  const clock = mock.clock(on, { now: 1_000_000 })
  on('fs.exists', (_: unknown, e: { path: string }) => ({ value: !e.path.endsWith('/.git') || e.path === '/repo/.git' }))
  const files: Record<string, string> = { '/proc/self/stat': '4242 (claude) S 1 4242', '/repo/.claude/settings.local.json': localSettings }
  on('fs.read', (_: unknown, e: { path: string }) => ({ value: files[e.path] ?? index }))
  on('fs.write', () => ({ value: undefined }))
  on('command.register', () => ({ value: {} }))
  on('ui.open', () => ({ value: { isPlaced: true } }))
  on('ui.close', () => ({ value: undefined }))
  on('session.start', (_: unknown, e: { cwd: string }) => ({ cwd: e.cwd }))
  on('prompt.submit', (_: unknown, e: { text: string }) => ({ text: e.text }))
  await $.session.start({ cwd: '/repo', surface: 'terminal', isInteractive: true })
  return clock
}

test('a dictation from this pane becomes a note', async ($, on) => {
  const clock = await start($, on, row(4242, 'the weakest part is the sync module') + '\n')
  await clock.advance(1100)
  const ui = await $.ui.mount({ ...PANE, surface: 'terminal' })
  expect((await ui.find({ type: 'Text', text: /sync module/ }))?.text).toContain('t1')
  await ui.unmount()
})

test('a dictation from another session stays out', async ($, on) => {
  const clock = await start($, on, row(9999, 'talking to the dev agent') + '\n')
  await clock.advance(1100)
  const ui = await $.ui.mount({ ...PANE, surface: 'terminal' })
  expect(await ui.find({ type: 'Text', text: /dev agent/ })).toBeUndefined()
  await ui.unmount()
})

const BAND = { plugin: 'voice-notes', component: 'AbovePrompt', props: { hasSurvey: false } } as const

test('minimize leaves a one-line summary above the prompt', async ($, on) => {
  const clock = await start($, on, row(4242, 'the weakest part is the sync module') + '\n')
  await clock.advance(1100)
  const pane = await $.ui.mount({ ...PANE, surface: 'terminal' })
  await pane.press({ key: 'min' })
  await pane.unmount()
  const band = await $.ui.mount({ ...BAND, surface: 'terminal' })
  expect((await band.find({ type: 'Text', text: /notes ·/ }))?.text).toBe('1 notes · no proposal')
  await band.unmount()
})

const asked = (q: string): object => ({ claims: [], question: q })
const reply = (span: string): string => JSON.stringify({ fields: {
  weakest_part: { claims: [{ id: 'c1', text: 'The sync module.', said: [span] }] },
  force: asked('f?'), smallest_move: asked('m?'), preserves: asked('p?'), would_reverse_it: asked('r?'),
} })

async function placeWith($: any, on: any, text: string): Promise<any> {
  on('model.complete', (_: unknown, e: { model: string; system: string }) => ({ value: { isAnswered: true, text, usage: { inputTokens: 1, outputTokens: 1 }, model: e.model } }))
  const clock = await start($, on, row(4242, 'the weakest part is the sync module') + '\n')
  await clock.advance(1100)
  const pane = await $.ui.mount({ ...PANE, surface: 'terminal' })
  await pane.press({ key: 'place' })
  return pane
}

test('a reply quoting the notes becomes a proposal', async ($, on) => {
  const pane = await placeWith($, on, reply('the sync module'))
  expect((await pane.find({ type: 'Text', text: /said t1/ }))?.text).toBe('said t1')
  await pane.unmount()
})

test('a reply quoting words never said is refused', async ($, on) => {
  const pane = await placeWith($, on, reply('the sync layer'))
  expect((await pane.find({ type: 'Text', text: /broke the contract/ }))?.text).toContain('span_not_in_transcript')
  await pane.unmount()
})

const TAP = JSON.stringify({ voice: { mode: 'tap' } })

test('in tap mode a dictation that auto-submits is kept as a note and not sent', async ($, on) => {
  const clock = await start($, on, row(4242, 'the weakest part is the sync module') + '\n', TAP)
  await clock.advance(1100)
  const result = await $.prompt.submit({ text: 'The weakest part is the sync module' })
  expect(result).toEqual({ drop: 'Kept as voice note t1; not sent to the agent (tap mode).' })
})

test('in tap mode a typed prompt still reaches the agent', async ($, on) => {
  const clock = await start($, on, row(4242, 'the weakest part is the sync module') + '\n', TAP)
  await clock.advance(1100)
  const pending = $.prompt.submit({ text: 'run the tests' })
  await clock.advance(5000)
  expect((await pending).drop).toBeUndefined()
})

test('in hold mode a dictation sent with Enter reaches the agent', async ($, on) => {
  const clock = await start($, on, row(4242, 'the weakest part is the sync module') + '\n')
  await clock.advance(1100)
  const result = await $.prompt.submit({ text: 'the weakest part is the sync module' })
  expect(result.drop).toBeUndefined()
})

test('in tap mode the dropped dictation is cleared from the prompt box', async ($, on) => {
  const filled: string[] = []
  on('prompt.read', () => ({ value: { text: 'The weakest part is the sync module', cursor: 35 } }))
  on('prompt.fill', (_: unknown, e: { text: string }) => {
    filled.push(e.text)
    return { isFilled: true }
  })
  const clock = await start($, on, row(4242, 'the weakest part is the sync module') + '\n', TAP)
  await clock.advance(1100)
  await $.prompt.submit({ text: 'The weakest part is the sync module' })
  await clock.advance(500)
  expect(filled).toEqual([''])
})

test('in tap mode a box edited since the drop is left alone', async ($, on) => {
  const filled: string[] = []
  on('prompt.read', () => ({ value: { text: 'now I am typing', cursor: 15 } }))
  on('prompt.fill', (_: unknown, e: { text: string }) => {
    filled.push(e.text)
    return { isFilled: true }
  })
  const clock = await start($, on, row(4242, 'the weakest part is the sync module') + '\n', TAP)
  await clock.advance(1100)
  await $.prompt.submit({ text: 'The weakest part is the sync module' })
  await clock.advance(500)
  expect(filled).toEqual([])
})
