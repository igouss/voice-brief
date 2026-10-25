import { atom, read, update } from 'claude-code'
import type { EngineInterface, Register, RenderSurface } from 'claude-code'

import type { Claim, Field, Note, PaneSize, Proposal, ThreadEvent, ThreadLine } from '../types'
import { FIELDS as KEYS, TITLES, validate } from './contract'

const PANE = 'voice-notes'
const FIELDS: [Field, string][] = KEYS.map(k => [k, TITLES[k]])

const notes = atom({ plugin: 'voice-notes', key: 'notes' } as const, [] as Note[], { shape: 'notes-1' })
const proposal = atom({ plugin: 'voice-notes', key: 'proposal' } as const, null as Proposal | null, { shape: 'proposal-1' })
const accepted = atom({ plugin: 'voice-notes', key: 'accepted' } as const, null as Proposal | null, { shape: 'proposal-1' })
const rev = atom({ plugin: 'voice-notes', key: 'rev' } as const, 0)
const busy = atom({ plugin: 'voice-notes', key: 'busy' } as const, null as string | null)
const message = atom({ plugin: 'voice-notes', key: 'message' } as const, '')
const latest = atom({ plugin: 'voice-notes', key: 'latest' } as const, null as string | null)
const size = atom({ plugin: 'voice-notes', key: 'size' } as const, 'normal' as PaneSize)

const COLUMNS: Record<PaneSize, number> = { min: 0, normal: 56, max: 400 }
const PROXY_URL = 'http://127.0.0.1:8897'
const MODEL = 'claude-opus-5-5'
const EFFORT = 'medium'

type IndexRow = { name: string; start: number; pid?: number | null; final: string; wav: string; closed_by_client: boolean; endpoint: boolean }

const norm = (s: string): string => s.replace(/[‘’]/g, "'").split(/\s+/).filter(Boolean).join(' ').toLowerCase()
const tid = (i: number): string => `t${i + 1}`

let isCapturing = false
let pid = 0
let home = ''
let dataHome = ''
let projectData = ''
let project = ''
let work = ''
let startedSecs = 0
let thread: ThreadLine[] = []

async function log($: EngineInterface, event: ThreadEvent): Promise<void> {
  const line: ThreadLine = { n: thread.length + 1, at: await $.clock.now(), ...event }
  thread = [...thread, line]
  await $.fs.write(`${work}/thread.jsonl`, thread.map(e => JSON.stringify(e)).join('\n') + '\n')
}

const live = (list: Note[]): [string, Note][] =>
  list.map((n, i): [string, Note] => [tid(i), n]).filter(([, n]) => !n.deleted)

async function poll($: EngineInterface): Promise<void> {
  const index = `${projectData}/index.jsonl`
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
  for (const n of added) await log($, { type: 'TakeRecorded', take: tid(count - added.length + added.indexOf(n)), stream: n.name, text: n.text, wav: n.wav, ok: n.ok })
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
  const dir = `${work}/place-${thread.length + 1}`
  const transcript = takes.map(([t, note]) => `[${t}] ${note.text}`).join('\n\n')
  await update($, busy, () => `Placing ${takes.map(([t]) => t).join(', ')}…`)
  await update($, message, () => '')
  const base = await read($, rev)
  await log($, { type: 'JobSubmitted', base, ctx: takes.map(([t]) => t), dir })
  const t0 = await $.clock.now()
  try {
    const system = await $.fs.read(`${$.plugin.root}/prompt/system.md`)
    await $.fs.write(`${dir}/system.md`, system)
    await $.fs.write(`${dir}/transcript.txt`, transcript)
    const r = await $.model.complete({ model: MODEL, system, prompt: transcript, effort: EFFORT, timeoutMs: 300000 })
    const secs = Math.round(((await $.clock.now()) - t0) / 100) / 10
    await $.fs.write(`${dir}/call.json`, JSON.stringify({ model: MODEL, effort: EFFORT, secs, isAnswered: r.isAnswered, usage: r.usage }, null, 1))
    if (!r.isAnswered) {
      await log($, { type: 'JobFailed', reason: r.reason })
      await update($, message, () => `The model gave no reply (${r.reason}).`)
      return
    }
    await $.fs.write(`${dir}/reply.txt`, r.text)
    const v = validate(transcript, r.text)
    if (!v.ok) {
      await log($, { type: 'JobFailed', violations: v.violations })
      await update($, message, () => `The reply broke the contract: ${v.violations.map(x => x.violation).join(', ')}`)
      return
    }
    const claims: Proposal['claims'] = {}
    const asked: Proposal['asked'] = {}
    for (const key of KEYS) {
      const placement = v.placed[key]
      if (placement.kind === 'claims') claims[key] = placement.claims
      else asked[key] = placement.question
    }
    const p: Proposal = { base, ctx: takes.map(([t]) => t), claims, asked, secs }
    await update($, proposal, () => p)
    await log($, { type: 'ProposalMade', kind: 'place', secs, claims, asked })
    await update($, message, () => `Proposal ready in ${secs}s. Accept it, or dictate an answer and place again.`)
  } catch (err) {
    await log($, { type: 'JobFailed', error: String(err) })
    await update($, message, () => String(err))
  } finally {
    await update($, busy, () => null)
  }
}

async function gitRoot($: EngineInterface, cwd: string): Promise<string> {
  let dir = cwd
  while (dir !== '' && dir !== '/') {
    if (await $.fs.exists(`${dir}/.git`)) return dir
    dir = dir.slice(0, dir.lastIndexOf('/'))
  }
  return cwd
}

async function setup($: EngineInterface): Promise<string> {
  const mitmdump = `${home}/.local/bin/mitmdump`
  if (!(await $.fs.exists(mitmdump))) return 'mitmdump is not at ~/.local/bin/mitmdump. Run `uv tool install mitmproxy`, then /vb-setup again.'
  const addon = `${dataHome}/voice-brief/vb_mitm.py`
  await $.fs.write(addon, await $.fs.read(`${$.plugin.root}/setup/vb_mitm.py`))
  const config = (await $.env.get('XDG_CONFIG_HOME')) ?? `${home}/.config`
  const unit = `${config}/systemd/user/vb-voice-proxy.service`
  await $.fs.write(unit, (await $.fs.read(`${$.plugin.root}/setup/vb-voice-proxy.service.in`)).replaceAll('@ADDON@', addon))
  for (const argv of [['systemctl', '--user', 'daemon-reload'], ['systemctl', '--user', 'enable', 'vb-voice-proxy.service'], ['systemctl', '--user', 'restart', 'vb-voice-proxy.service']]) {
    const r = await $.process.run(argv)
    if (r.exitCode !== 0) return `\`${argv.join(' ')}\` failed: ${r.stderr.trim()}`
  }
  const ca = `${home}/.mitmproxy/mitmproxy-ca-cert.pem`
  for (let i = 0; i < 50 && !(await $.fs.exists(ca)); i++) await $.clock.sleep(100)
  if (!(await $.fs.exists(ca))) return `The proxy started but ${ca} did not appear. See \`journalctl --user -u vb-voice-proxy\`.`
  const path = `${project}/.claude/settings.local.json`
  const current: { env?: Record<string, string> } = (await $.fs.exists(path)) ? JSON.parse(await $.fs.read(path)) : {}
  current.env = { ...(current.env ?? {}), HTTPS_PROXY: PROXY_URL, NODE_EXTRA_CA_CERTS: ca }
  await $.fs.write(path, JSON.stringify(current, null, 2) + '\n')
  const ignore = (await $.fs.exists(`${project}/.gitignore`)) ? await $.fs.read(`${project}/.gitignore`) : ''
  const warn = ignore.includes('settings.local.json') ? '' : ' Add `.claude/settings.local.json` to .gitignore: it holds a path under your home.'
  await log($, { type: 'SetUp', unit, addon, settings: path })
  return `Installed ${unit} (proxy on ${PROXY_URL}) and wrote ${path}. Restart Claude Code in ${project} to start capturing.${warn}`
}

async function resize($: EngineInterface, to: PaneSize): Promise<void> {
  await update($, size, () => to)
  await $.ui.close({ id: PANE })
  if (to !== 'min') await $.ui.open({ id: PANE, title: 'Voice notes', columns: COLUMNS[to], rows: to === 'max' ? 40 : 16 })
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
  await log($, { type: 'Accepted', revision: now })
  await update($, message, () => `Accepted as revision ${now}. Hand off when it says what you mean.`)
}

async function reject($: EngineInterface): Promise<void> {
  if (!(await read($, proposal))) return
  await update($, proposal, () => null)
  await log($, { type: 'Rejected' })
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
  await log($, { type: 'HandedOff', target: 'clipboard', revision: now, path: `${work}/prompt.md` })
  await update($, message, () => `Revision ${now} copied to the clipboard and written to ${work}/prompt.md`)
}

async function remove($: EngineInterface, take: string): Promise<void> {
  const i = Number(take.replace(/^t/, '')) - 1
  await update($, notes, list => list.map((n, j) => (j === i ? { ...n, deleted: true } : n)))
  await log($, { type: 'TakeDeleted', take })
  await update($, message, () => `${take} deleted. Place again.`)
}

async function fix($: EngineInterface, take: string, text: string): Promise<void> {
  const i = Number(take.replace(/^t/, '')) - 1
  const before = (await read($, notes))[i]?.text
  if (before === undefined || !text.trim()) return
  await update($, notes, list => list.map((n, j) => (j === i ? { ...n, text } : n)))
  await log($, { type: 'TranscriptCorrected', take, before, after: text })
  await update($, message, () => `${take} corrected. Place again.`)
}

export const register: Register = on => {
  on('session.start', async ($, e, next) => {
    home = (await $.env.get('HOME')) ?? ''
    dataHome = (await $.env.get('XDG_DATA_HOME')) ?? `${home}/.local/share`
    isCapturing = (await $.env.get('HTTPS_PROXY')) === PROXY_URL
    project = await gitRoot($, e.cwd)
    projectData = `${dataHome}/voice-brief/projects/${project.replaceAll('/', '-')}`
    pid = Number((await $.fs.read('/proc/self/stat')).split(' ')[0])
    startedSecs = (await $.clock.now()) / 1000 - 5
    work = `${projectData}/notes/${Math.round(startedSecs)}`
    thread = []
    await log($, { type: 'SessionStarted', pid, project, isCapturing })
    for (const name of ['vb', 'vb-min', 'vb-max', 'vb-place', 'vb-accept', 'vb-reject', 'vb-handoff', 'vb-delete', 'vb-fix', 'vb-setup']) {
      await $.command.register({ name, description: `voice-brief: ${name === 'vb' ? 'open the notes pane' : name.slice(3)}` })
    }
    $.clock.every(1000, () => void poll($))
    $.ui.status(isCapturing ? 'voice notes: listening' : 'voice notes: not capturing (/vb-setup)')
    void resize($, await read($, size))
    return next(e)
  })

  on('command.run', async ($, e, next) => {
    if (!e.command.startsWith('vb')) return next(e)
    const [take = '', ...rest] = e.args.trim().split(/\s+/)
    switch (e.command) {
      case 'vb': await resize($, 'normal'); return { text: 'Voice notes pane opened.' }
      case 'vb-min': await resize($, 'min'); return { text: 'Voice notes minimized to the line above the prompt.' }
      case 'vb-max': await resize($, 'max'); return { text: 'Voice notes maximized.' }
      case 'vb-place': await place($); return { text: (await read($, message)) || 'Placed.' }
      case 'vb-accept': await accept($); return { text: await read($, message) }
      case 'vb-reject': await reject($); return { text: await read($, message) }
      case 'vb-handoff': await handOff($); return { text: await read($, message) }
      case 'vb-delete': await remove($, take); return { text: await read($, message) }
      case 'vb-fix': await fix($, take, rest.join(' ')); return { text: await read($, message) }
      case 'vb-setup': return { text: await setup($) }
      default: return next(e)
    }
  })

  on('prompt.submit', async ($, e, next) => {
    const text = norm(e.text)
    const list = await read($, notes)
    const sent = list.map((n, i) => (!n.sent && norm(n.text) && text.includes(norm(n.text)) ? i : -1)).filter(i => i >= 0)
    if (sent.length) {
      await update($, notes, l => l.map((n, i) => (sent.includes(i) ? { ...n, sent: true } : n)))
      for (const i of sent) await log($, { type: 'TakeSent', take: tid(i) })
    }
    await update($, latest, () => null)
    return next(e)
  })

  on('ui.render', { component: 'AbovePrompt' }, async ($, e, next) => {
    const last = await read($, latest)
    const isMin = (await read($, size)) === 'min'
    if (!last && !isMin) return next(e)
    const { Box, Text, Button } = $.ui.resolve(e)
    const list = await read($, notes)
    const p = await read($, proposal)
    const acc = await read($, accepted)
    const working = await read($, busy)
    const state = working ?? (p ? 'proposal ready' : acc ? `revision ${await read($, rev)} accepted` : 'no proposal')
    return (
      <Box flexDirection="column">
        {last && (
          <Box>
            <Text color="success">● captured </Text>
            <Text wrap="truncate-end">{last}</Text>
            <Text dimColor>   saved to notes; Enter also sends it to Claude</Text>
          </Box>
        )}
        {isMin && (
          <Box gap={1}>
            <Text color="claude">Voice notes</Text>
            <Text dimColor>{live(list).length} notes · {state}</Text>
            <Button key="restore" label="Open" onPress={() => void resize($, 'normal')} />
            <Button key="max" label="Maximize" onPress={() => void resize($, 'max')} />
          </Box>
        )}
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
        <Box gap={1}>
          <Button key="min" label="Minimize" onPress={() => void resize($, 'min')} />
          {(await read($, size)) === 'max'
            ? <Button key="restore" label="Restore" onPress={() => void resize($, 'normal')} />
            : <Button key="max" label="Maximize" onPress={() => void resize($, 'max')} />}
        </Box>
        <Box flexDirection="column">
          <Text bold>Notes ({live(list).length}) <Text dimColor>{project}</Text></Text>
          {!isCapturing && <Text color="warning">Not capturing: this session does not go through the voice proxy. Run /vb-setup, then restart Claude Code.</Text>}
          {isCapturing && list.length === 0 && <Text dimColor>Dictate with /voice. Each dictation lands here; Enter is optional.</Text>}
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
