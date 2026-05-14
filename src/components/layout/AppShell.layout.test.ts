import { describe, expect, it } from 'vitest'
import css from '../../index.css?raw'

function getRuleBody(selector: string) {
  const escapedSelector = selector.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')
  const match = css.match(new RegExp(`${escapedSelector}\\s*\\{([^}]*)\\}`))
  return match?.[1] ?? ''
}

describe('AppShell layout styles', () => {
  it('places the titlebar above the sidebar/content body', () => {
    const appRule = getRuleBody('.app')
    const bodyRule = getRuleBody('.app-body')

    expect(appRule).toContain('flex-direction: column;')
    expect(bodyRule).toContain('display: flex;')
    expect(bodyRule).toContain('min-height: 0;')
  })
})
