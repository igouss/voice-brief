import { atom, read, update } from 'claude-code'
import type { EngineInterface, Register, RenderSurface } from 'claude-code'

import type { Claim, Note, Proposal } from '../types'

const PANE = 'voice-notes'
const FIELDS: [string, string][] = [
  ['weakest_part', 'Weakest part'],
  ['force', 'Force'],
  ['smallest_move', 'Smallest move'],
  ['preserves', 'Preserves'],
  ['would_reverse_it', 'Would reverse it'],
]

const notes = atom({ plugin: 'voice-notes', key: 'notes' } as const, [] as Note[], { shape: 'notes-1' })
const proposal = atom({ plugin: 'voice-notes', key: 'proposal' } as const, null as Proposal | null, { shape: 'proposal-1' })
const accepted = atom({ plugin: 'voice-notes', key: 'accepted' } as const, null as Proposal | null, { shape: 'proposal-1' })
const rev = atom({ plugin: 'voice-notes', key: 'rev' } as const, 0)
const busy = atom({ plugin: 'voice-notes', key: 'busy' } as const, null as string | null)
const message = atom({ plugin: 'voice-notes', key: 'message' } as const, '')
const latest = atom({ plugin: 'voice-notes', key: 'latest' } as const, null as string | null)

type IndexRow = { name: string; start: number; pid?: number | null; final: string; wav: string; closed_by_client: boolean; endpoint: boolean }

const norm = (s: string): string => s.replace(/[‘’]/g, "'").split(/\s+/).filter(Boolean).join(' ').toLowerCase()
const tid = (i: number): string => `t${i + 1}`

let isNotesPane = false
let pid = 0
let home = ''
let root = ''
let work = ''
let startedSecs = 0
let thread: object[] = []

async function log($: EngineInterface, type: string, data: object): Promise<void> {
  thread = [...thread, { n: thread.length + 1, type, at: await $.clock.now(), ...data }]
  await $.fs.write(`${work}/thread.jsonl`, thread.map(e => JSON.stringify(e)).join('\n') + '\n')
}

const live = (list: Note[]): [string, Note][] =>
  list.map((n, i): [string, Note] => [tid(i), n]).filter(([, n]) => !n.deleted)

async function poll($: EngineInterface): Promise<void> {
  const index = `${home}/tmp/vb-proto-audio/index.jsonl`
  if (!(await $.fs.exists(index))) return
  const rows: IndexRow[] = (await $.fs.read(index))
    .split('\n')
    .filter((l: string) => l.trim())
    .map((l: string) => JSON.parse(l))
    .filter((r: IndexRow) => r.pid === pid && r.start >= startedSecs && r.final.trim())
  const known = new Set((await read($, notes)).map(n => n.name))
  const fresh = rows.filter(r => !known.has(r.name))
  if (fresh.length === 0) return
  const added: Note[] = fresh.map(r => ({
    name: r.name, startMs: Math.round(r.start * 1000), text: r.final, wav: r.wav,
    ok: r.closed_by_client && r.endpoint, sent: false, deleted: false,
  }))
  await update($, notes, list => [...list, ...added])
  const count = (await read($, notes)).length
  for (const n of added) await log($, 'TakeRecorded', { take: tid(count - added.length + added.indexOf(n)), stream: n.name, text: n.text, wav: n.wav, ok: n.ok })
  const newest: Note | undefined = added[added.length - 1]
  if (newest) await update($, latest, () => `${tid(count - 1)}: ${newest.text}`)
  $.ui.status(`voice notes: ${count} captured`)
  $.ui.toast(`Voice note ${tid(count - 1)} captured`)
}

async function place($: EngineInterface): Promise<void> {
  const takes = live(await read($, notes))
  if (takes.length === 0) {
    await update($, message, () => 'No notes yet: dictate with /voice first.')
    return
  }
  const n = thread.length + 1
  const dir = `${work}/place-${n}`
  const transcript = takes.map(([t, note]) => `[${t}] ${note.text}`).join('\n\n')
  await $.fs.write(`${dir}/transcript.txt`, transcript)
  await update($, busy, () => `Placing ${takes.map(([t]) => t).join(', ')}…`)
  await update($, message, () => '')
  const base = await read($, rev)
  await log($, 'JobSubmitted', { base, ctx: takes.map(([t]) => t), dir })
  const t0 = await $.clock.now()
  try {
    const render = await $.process.run([`${root}/spike/contract/target/debug/render`, `${root}/idea.md`, `${root}/spike/contract/prompt/system.md`])
    await $.fs.write(`${dir}/system.md`, render.stdout)
    const call = await $.process.run([`${root}/spike/isolation/run.sh`, 'isolated', `${dir}/system.md`, `${dir}/transcript.txt`, dir], { timeoutMs: 300000, stdin: '' })
    if (call.exitCode !== 0) throw new Error(`model call failed (${call.exitCode}): ${call.stderr.slice(-300)}`)
    const v = await $.process.run([`${root}/spike/contract/target/debug/validate`, 'notes', `${dir}/transcript.txt`, `${dir}/reply.txt`])
    const secs = Math.round(((await $.clock.now()) - t0) / 100) / 10
    if (v.exitCode !== 0) {
      await log($, 'JobFailed', { violations: v.stdout.trim().split('\n') })
      await update($, message, () => `The reply broke the contract: ${v.stdout.trim().slice(0, 300)}`)
      return
    }
    const events = JSON.parse(v.stdout)
    const asked: Record<string, string> = {}
    for (const e of events.slice(1)) asked[e.field] = e.question
    const p: Proposal = { base, ctx: takes.map(([t]) => t), claims: events[0].fields, asked, secs }
    await update($, proposal, () => p)
    await log($, 'ProposalMade', { kind: 'place', secs, claims: p.claims, asked })
    await update($, message, () => `Proposal ready in ${secs}s. Accept it, or dictate an answer and place again.`)
  } catch (err) {
    await log($, 'JobFailed', { error: String(err) })
    await update($, message, () => String(err))
  } finally {
    await update($, busy, () => null)
  }
}

async function isFresh($: EngineInterface, p: Proposal): Promise<boolean> {
  const ids = live(await read($, notes)).map(([t]) => t)
  return p.base === (await read($, rev)) && p.ctx.every(t => ids.includes(t))
}

async function accept($: EngineInterface): Promise<void> {
  const p = await read($, proposal)
  if (!p) return
  if (!(await isFresh($, p))) {
    await update($, message, () => 'Stale: a note it used was deleted. Place again.')
    return
  }
  await update($, rev, r => r + 1)
  await update($, accepted, () => p)
  await update($, proposal, () => null)
  const now = await read($, rev)
  await log($, 'Accepted', { revision: now })
  await update($, message, () => `Accepted as revision ${now}. Hand off when it says what you mean.`)
}

async function reject($: EngineInterface): Promise<void> {
  if (!(await read($, proposal))) return
  await update($, proposal, () => null)
  await log($, 'Rejected', {})
  await update($, message, () => 'Rejected.')
}

const promptMd = (p: Proposal): string =>
  FIELDS.map(([key, title]) => {
    const claims: Claim[] = p.claims[key] ?? []
    const body = claims.length ? claims.map(c => `- ${c.text}`).join('\n') : `(open: ${p.asked[key] ?? ''})`
    return `## ${title}\n\n${body}`
  }).join('\n\n') + '\n'

async function handOff($: EngineInterface, surface?: RenderSurface): Promise<void> {
  const p = await read($, accepted)
  if (!p) {
    await update($, message, () => 'Accept a proposal first.')
    return
  }
  const text = promptMd(p)
  await $.fs.write(`${work}/prompt.md`, text)
  $.ui.copy({ text, surface })
  const now = await read($, rev)
  await log($, 'HandedOff', { target: 'clipboard', revision: now, path: `${work}/prompt.md` })
  await update($, message, () => `Revision ${now} copied to the clipboard and written to ${work}/prompt.md`)
}

async function remove($: EngineInterface, take: string): Promise<void> {
  const i = Number(take.replace(/^t/, '')) - 1
  await update($, notes, list => list.map((n, j) => (j === i ? { ...n, deleted: true } : n)))
  await log($, 'TakeDeleted', { take })
  await update($, message, () => `${take} deleted. Place again.`)
}

async function fix($: EngineInterface, take: string, text: string): Promise<void> {
  const i = Number(take.replace(/^t/, '')) - 1
  const before = (await read($, notes))[i]?.text
  if (before === undefined || !text.trim()) return
  await update($, notes, list => list.map((n, j) => (j === i ? { ...n, text } : n)))
  await log($, 'TranscriptCorrected', { take, before, after: text })
  await update($, message, () => `${take} corrected. Place again.`)
}

export const register: Register = on => {
  on('session.start', async ($, e, next) => {
    isNotesPane = (await $.env.get('VB_BRIEF')) === '1'
    if (!isNotesPane) return next(e)
    home = (await $.env.get('HOME')) ?? ''
    root = e.cwd
    const ppid = await $.process.run(['sh', '-c', 'p=$PPID; while [ "$p" -gt 1 ]; do [ "$(cat /proc/$p/comm)" = claude ] && { echo $p; exit; }; p=$(cut -d" " -f4 /proc/$p/stat); done; echo $PPID'])
    pid = Number(ppid.stdout.trim())
    startedSecs = (await $.clock.now()) / 1000 - 5
    work = `${home}/tmp/vb-notes/${Math.round(startedSecs)}`
    thread = []
    await log($, 'SessionStarted', { pid, root })
    await $.process.run(['cargo', 'build', '-q', '--bins'], { cwd: `${root}/spike/contract`, timeoutMs: 300000 })
    for (const name of ['vb', 'vb-place', 'vb-accept', 'vb-reject', 'vb-handoff', 'vb-delete', 'vb-fix']) {
      await $.command.register({ name, description: `voice-brief: ${name === 'vb' ? 'open the notes pane' : name.slice(3)}` })
    }
    $.clock.every(1000, () => void poll($))
    $.ui.status('voice notes: listening')
    void $.ui.open({ id: PANE, title: 'Voice notes' })
    return next(e)
  })

  on('command.run', async ($, e, next) => {
    if (!isNotesPane || !e.command.startsWith('vb')) return next(e)
    const [take = '', ...rest] = e.args.trim().split(/\s+/)
    switch (e.command) {
      case 'vb': await $.ui.open({ id: PANE, title: 'Voice notes' }); return { text: 'Voice notes pane opened.' }
      case 'vb-place': await place($); return { text: (await read($, message)) || 'Placed.' }
      case 'vb-accept': await accept($); return { text: await read($, message) }
      case 'vb-reject': await reject($); return { text: await read($, message) }
      case 'vb-handoff': await handOff($); return { text: await read($, message) }
      case 'vb-delete': await remove($, take); return { text: await read($, message) }
      case 'vb-fix': await fix($, take, rest.join(' ')); return { text: await read($, message) }
      default: return next(e)
    }
  })

  on('prompt.submit', async ($, e, next) => {
    if (!isNotesPane) return next(e)
    const text = norm(e.text)
    const list = await read($, notes)
    const sent = list.map((n, i) => (!n.sent && norm(n.text) && text.includes(norm(n.text)) ? i : -1)).filter(i => i >= 0)
    if (sent.length) {
      await update($, notes, l => l.map((n, i) => (sent.includes(i) ? { ...n, sent: true } : n)))
      for (const i of sent) await log($, 'TakeSent', { take: tid(i) })
    }
    await update($, latest, () => null)
    return next(e)
  })

  on('ui.render', { component: 'AbovePrompt' }, async ($, e, next) => {
    const last = isNotesPane ? await read($, latest) : null
    if (!last) return next(e)
    const { Box, Text } = $.ui.resolve(e)
    return (
      <Box>
        <Text color="success">● captured </Text>
        <Text wrap="truncate-end">{last}</Text>
        <Text dimColor>   saved to notes; Enter also sends it to Claude</Text>
      </Box>
    )
  })

  on('ui.render', { component: 'Pane', requestId: PANE }, async ($, e) => {
    const { Box, Text, Button } = $.ui.resolve(e)
    const list = await read($, notes)
    const p = await read($, proposal)
    const acc = await read($, accepted)
    const r = await read($, rev)
    const working = await read($, busy)
    const msg = await read($, message)
    const view = p ?? acc
    const stale = p ? !(await isFresh($, p)) : false
    const takeOf = (span: string): string => {
      const s = norm(span)
      const i = list.findIndex(n => !n.deleted && norm(n.text).includes(s))
      return i >= 0 ? tid(i) : '?'
    }
    return (
      <Box flexDirection="column" gap={1}>
        <Box flexDirection="column">
          <Text bold>Notes ({live(list).length})</Text>
          {list.length === 0 && <Text dimColor>Dictate with /voice. Each dictation lands here; Enter is optional.</Text>}
          {list.map((n, i) => (
            <Box key={n.name}>
              <Text dimColor={n.deleted} strikethrough={n.deleted}>
                <Text bold>{tid(i)}</Text> {n.sent ? '↗ ' : ''}{n.text}
              </Text>
              {!n.deleted && <Button key={`del-${n.name}`} label="×" onPress={() => void remove($, tid(i))} />}
            </Box>
          ))}
        </Box>
        <Box gap={1}>
          <Button key="place" variant="primary" label="Place" onPress={() => void place($)} />
          {p && <Button key="accept" label="Accept" onPress={() => void accept($)} />}
          {p && <Button key="reject" label="Reject" onPress={() => void reject($)} />}
          {acc && <Button key="handoff" label="Hand off" onPress={press => void handOff($, press.surface)} />}
        </Box>
        {working && <Text color="warning">{working}</Text>}
        {msg !== '' && <Text dimColor>{msg}</Text>}
        {view && (
          <Box flexDirection="column">
            <Text bold>{p ? `Proposal from ${p.ctx.join(', ')}${stale ? '  (stale)' : ''}` : `Accepted, revision ${r}`}</Text>
            {FIELDS.map(([key, title]) => (
              <Box key={key} flexDirection="column" marginTop={1}>
                <Text color="claude">{title}</Text>
                {(view.claims[key] ?? []).map(c => (
                  <Box key={c.id} flexDirection="column" paddingLeft={2}>
                    <Text>{c.text}</Text>
                    <Text dimColor>{c.said ? `said ${[...new Set(c.said.map(takeOf))].join(', ')}` : `added: ${c.added}`}</Text>
                  </Box>
                ))}
                {view.asked[key] && <Text color="warning">  ? {view.asked[key]}</Text>}
              </Box>
            ))}
          </Box>
        )}
      </Box>
    )
  })
}
