import { describe, expect, it } from 'vitest'
import css from '../index.css?raw'

function getRuleBody(selector: string) {
  const escapedSelector = selector.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')
  const match = css.match(new RegExp(`${escapedSelector}\\s*\\{([^}]*)\\}`))
  return match?.[1] ?? ''
}

describe('settings mirror layout styles', () => {
  it('keeps mirror option dropdowns visible in narrow settings windows', () => {
    const configGridRule = getRuleBody('.grid4.config-grid')
    const configCellRule = getRuleBody('.config-grid > .col')
    const dropdownRule = getRuleBody('.dd')

    expect(configGridRule).toContain('grid-template-columns: repeat(auto-fit, minmax(132px, 1fr));')
    expect(configGridRule).not.toContain('repeat(4, minmax(150px, 1fr))')
    expect(configCellRule).toContain('min-width: 0;')
    expect(dropdownRule).toContain('width: 100%;')
    expect(dropdownRule).toContain('min-width: 0;')
  })
})
