import { expect, test } from 'claude-code/testing'

import { validate } from '../hooks/contract'
import cases from './contract-cases'

for (const c of cases.cases) {
  test(c.name, () => {
    const r = validate(cases.transcript, c.reply)
    expect(r.ok ? 'ok' : r.violations.map(v => v.violation)).toEqual(c.expect)
  })
}
