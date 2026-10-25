import type { Claim, Field } from '../types'

export type { Claim, Field }

export const TITLES: Record<Field, string> = {
  weakest_part: 'Weakest part',
  force: 'Force',
  smallest_move: 'Smallest move',
  preserves: 'Preserves',
  would_reverse_it: 'Would reverse it',
}

export const FIELDS: Field[] = Object.keys(TITLES) as Field[]

export type Placement = { kind: 'claims'; claims: Claim[] } | { kind: 'asked'; question: string }

export type Violation =
  | { violation: 'malformed'; detail: string }
  | { violation: 'unknown_field'; detail: string }
  | { violation: 'missing_field'; detail: Field }
  | { violation: 'empty_without_question'; detail: Field }
  | { violation: 'claims_and_question'; detail: Field }
  | { violation: 'claim_without_source'; detail: { field: Field; claim: string } }
  | { violation: 'claim_with_both_sources'; detail: { field: Field; claim: string } }
  | { violation: 'duplicate_claim_id'; detail: string }
  | { violation: 'empty_span'; detail: { field: Field; claim: string } }
  | { violation: 'span_not_in_transcript'; detail: { field: Field; claim: string; span: string } }

export type Placed = Record<Field, Placement>

export type Validated = { ok: true; placed: Placed } | { ok: false; violations: Violation[] }

type WireClaim = { id?: unknown; text?: unknown; said?: unknown; added?: unknown }
type WireField = { claims?: unknown; question?: unknown }

const isRecord = (v: unknown): v is Record<string, unknown> => typeof v === 'object' && v !== null && !Array.isArray(v)
const isField = (k: string): k is Field => (FIELDS as string[]).includes(k)

export function normalize(text: string): string {
  return text.replace(/[‘’]/g, "'").replace(/[“”]/g, '"').split(/\s+/).filter(Boolean).join(' ')
}

function shapeError(where: string): Validated {
  return { ok: false, violations: [{ violation: 'malformed', detail: where }] }
}

function wireClaims(field: WireField): WireClaim[] | null {
  const claims: unknown = field.claims ?? []
  if (!Array.isArray(claims) || !claims.every(isRecord)) return null
  return claims as WireClaim[]
}

export function validate(transcript: string, reply: string): Validated {
  let wire: unknown
  try {
    wire = JSON.parse(reply.trim())
  } catch (e) {
    return shapeError(String(e))
  }
  if (!isRecord(wire) || Object.keys(wire).some(k => k !== 'fields') || !isRecord(wire.fields)) {
    return shapeError('expected {"fields": {...}}')
  }
  const fields: Record<string, unknown> = wire.fields
  const haystack: string = normalize(transcript)
  const violations: Violation[] = []
  const ids = new Set<string>()
  const placed: Partial<Placed> = {}

  for (const key of Object.keys(fields)) {
    if (!isField(key)) violations.push({ violation: 'unknown_field', detail: key })
  }
  for (const field of FIELDS) {
    const raw: unknown = fields[field]
    if (raw === undefined) {
      violations.push({ violation: 'missing_field', detail: field })
      continue
    }
    if (!isRecord(raw) || Object.keys(raw).some(k => k !== 'claims' && k !== 'question')) {
      return shapeError(`field ${field}`)
    }
    const claims = wireClaims(raw)
    if (claims === null || (raw.question !== undefined && raw.question !== null && typeof raw.question !== 'string')) {
      return shapeError(`field ${field}`)
    }
    const question: string = typeof raw.question === 'string' ? raw.question.trim() : ''
    if (claims.length === 0 && question === '') {
      violations.push({ violation: 'empty_without_question', detail: field })
    } else if (claims.length === 0) {
      placed[field] = { kind: 'asked', question }
    } else if (question !== '') {
      violations.push({ violation: 'claims_and_question', detail: field })
    } else {
      const out: Claim[] = []
      for (const c of claims) {
        const keys = Object.keys(c)
        if (typeof c.id !== 'string' || typeof c.text !== 'string' || keys.some(k => !['id', 'text', 'said', 'added'].includes(k))) {
          return shapeError(`claim in ${field}`)
        }
        const said: unknown = c.said ?? []
        if (!Array.isArray(said) || !said.every(s => typeof s === 'string') || (c.added !== undefined && c.added !== null && typeof c.added !== 'string')) {
          return shapeError(`claim ${c.id} in ${field}`)
        }
        if (ids.has(c.id)) violations.push({ violation: 'duplicate_claim_id', detail: c.id })
        ids.add(c.id)
        const added: string | undefined = typeof c.added === 'string' ? c.added : undefined
        if (said.length === 0 && added === undefined) {
          violations.push({ violation: 'claim_without_source', detail: { field, claim: c.id } })
          continue
        }
        if (said.length > 0 && added !== undefined) {
          violations.push({ violation: 'claim_with_both_sources', detail: { field, claim: c.id } })
          continue
        }
        for (const span of said as string[]) {
          const needle: string = normalize(span)
          if (needle === '') violations.push({ violation: 'empty_span', detail: { field, claim: c.id } })
          else if (!haystack.includes(needle)) violations.push({ violation: 'span_not_in_transcript', detail: { field, claim: c.id, span } })
        }
        out.push(added !== undefined ? { id: c.id, text: c.text, added } : { id: c.id, text: c.text, said: said as string[] })
      }
      placed[field] = { kind: 'claims', claims: out }
    }
  }
  return violations.length === 0 ? { ok: true, placed: placed as Placed } : { ok: false, violations }
}
