import { describe, expect, it } from 'vitest'
import css from '../index.css?raw'

function getRuleBody(selector: string) {
  const escapedSelector = selector.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')
  const match = css.match(new RegExp(`${escapedSelector}\\s*\\{([^}]*)\\}`))
  return match?.[1] ?? ''
}

describe('LogsPage layout styles', () => {
  it('prevents horizontal scrollbar flicker in the log table', () => {
    const pageRule = getRuleBody('.logs-page')
    const tableRule = getRuleBody('.log-table')
    const bodyRule = getRuleBody('.log-table-body')
    const cellRule = getRuleBody('.log-row > *')
    const hoverRule = getRuleBody('.log-row:hover')

    expect(pageRule).toContain('display: flex;')
    expect(pageRule).toContain('min-height: 0;')
    expect(tableRule).toContain('min-height: 0;')
    expect(bodyRule).toContain('overflow-x: hidden;')
    expect(cellRule).toContain('min-width: 0;')
    expect(hoverRule).not.toContain('transform:')
  })
})
