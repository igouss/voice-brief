import { expect, mock, test } from 'claude-code/testing'

const PANE = {
  plugin: 'voice-notes',
  component: 'Pane',
  requestId: 'voice-notes',
  props: { title: 'Voice notes', isFocused: false, bodyColumns: 80, placement: 'dock', scroll: { offset: 0, bodyRows: 40 }, view: {} },
} as const

const row = (pid: number, final: string): string =>
  JSON.stringify({ name: `s-${pid}`, start: 1001, pid, final, wav: '/w.wav', closed_by_client: true, endpoint: true })

async function start($: any, on: any, index: string): Promise<{ advance: (ms: number) => Promise<void> }> {
  mock.env(on, { VB_BRIEF: '1', HOME: '/h' })
  const clock = mock.clock(on, { now: 1_000_000 })
  on('process.run', (_: unknown, e: { argv: string[] }) => ({ value: { exitCode: 0, stdout: e.argv[0] === 'sh' ? '4242\n' : '', stderr: '' } }))
  on('fs.exists', () => ({ value: true }))
  on('fs.read', () => ({ value: index }))
  on('fs.write', () => ({ value: undefined }))
  on('command.register', () => ({ value: {} }))
  on('ui.open', () => ({ value: { isPlaced: true } }))
  on('ui.close', () => ({ value: undefined }))
  on('session.start', (_: unknown, e: { cwd: string }) => ({ cwd: e.cwd }))
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
