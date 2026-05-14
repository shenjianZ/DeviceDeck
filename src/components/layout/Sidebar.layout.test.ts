import { describe, expect, it } from 'vitest'
import css from '../../index.css?raw'

function getRuleBody(selector: string) {
  const escapedSelector = selector.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')
  const match = css.match(new RegExp(`${escapedSelector}\\s*\\{([^}]*)\\}`))
  return match?.[1] ?? ''
}

describe('Sidebar layout styles', () => {
  it('uses normal text color for inactive navigation items', () => {
    const itemRule = getRuleBody('.sb-item')
    const activeRule = getRuleBody('.sb-item.active')

    expect(itemRule).toContain('color: var(--t1);')
    expect(itemRule).not.toContain('color: var(--t2);')
    expect(activeRule).toContain('color: var(--acc);')
  })
})
