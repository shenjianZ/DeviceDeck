import { render } from '@testing-library/react'
import { describe, expect, it } from 'vitest'
import '../i18n'
import css from '../index.css?raw'
import { SettingsPage } from './SettingsPage'

function getRuleBody(selector: string) {
  const escapedSelector = selector.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')
  const match = css.match(new RegExp(`${escapedSelector}\\s*\\{([^}]*)\\}`))
  return match?.[1] ?? ''
}

describe('settings appearance layout', () => {
  it('keeps language and font size dropdowns compact and right aligned', () => {
    const { container } = render(<SettingsPage />)
    const compactDropdowns = container.querySelectorAll('.settings-compact-select')
    const compactRule = getRuleBody('.settings-compact-select')

    expect(compactDropdowns).toHaveLength(2)
    expect(compactRule).toContain('width: 120px;')
    expect(compactRule).toContain('flex: 0 0 120px;')
    expect(compactRule).toContain('margin-left: auto;')
  })
})
