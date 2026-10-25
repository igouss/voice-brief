export type Note = { name: string; startMs: number; text: string; wav: string; ok: boolean; sent: boolean; deleted: boolean }
export type Claim = { id: string; text: string; said?: string[]; added?: string }
export type PaneSize = 'min' | 'normal' | 'max'
export type Proposal = { base: number; ctx: string[]; claims: Record<string, Claim[]>; asked: Record<string, string>; secs: number }

declare module 'claude-code' {
  interface PluginState {
    'voice-notes': {
      notes: Shaped<Note[]>
      proposal: Shaped<Proposal | null>
      accepted: Shaped<Proposal | null>
      rev: number
      busy: string | null
      message: string
      latest: string | null
      size: PaneSize
    }
  }
}
