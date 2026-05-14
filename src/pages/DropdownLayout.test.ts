import { describe, expect, it } from 'vitest'
import css from '../index.css?raw'
import logsPageSource from './LogsPage.tsx?raw'
import mirrorPageSource from './MirrorPage.tsx?raw'
import settingsPageSource from './SettingsPage.tsx?raw'

function getRuleBody(selector: string) {
  const escapedSelector = selector.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')
  const match = css.match(new RegExp(`${escapedSelector}\\s*\\{([^}]*)\\}`))
  return match?.[1] ?? ''
}

function count(source: string, token: string) {
  return source.split(token).length - 1
}

describe('dropdown layout contexts', () => {
  it('assigns every dropdown usage to a width context', () => {
    const totalDropdowns = count(logsPageSource + mirrorPageSource + settingsPageSource, '<Dropdown')
    const categorizedDropdowns =
      count(logsPageSource, 'className="toolbar-select"') +
      count(mirrorPageSource, 'className="device-select"') +
      count(settingsPageSource, 'className="settings-compact-select"') +
      count(settingsPageSource, 'className="settings-config-select"')

    expect(totalDropdowns).toBe(13)
    expect(categorizedDropdowns).toBe(totalDropdowns)
    expect(count(logsPageSource, 'className="toolbar-select"')).toBe(2)
    expect(count(mirrorPageSource, 'className="device-select"')).toBe(5)
    expect(count(settingsPageSource, 'className="settings-compact-select"')).toBe(2)
    expect(count(settingsPageSource, 'className="settings-config-select"')).toBe(4)
  })

  it('keeps compact dropdowns fixed and content dropdowns bounded by their container', () => {
    const toolbarRule = getRuleBody('.toolbar-select')
    const compactRule = getRuleBody('.settings-compact-select')
    const deviceRule = getRuleBody('.device-select')
    const configRule = getRuleBody('.settings-config-select')

    expect(toolbarRule).toContain('width: 124px;')
    expect(toolbarRule).toContain('flex: 0 0 124px;')
    expect(compactRule).toContain('width: 120px;')
    expect(compactRule).toContain('flex: 0 0 120px;')
    expect(deviceRule).toContain('width: 100%;')
    expect(deviceRule).toContain('min-width: 0;')
    expect(configRule).toContain('width: 100%;')
    expect(configRule).toContain('min-width: 0;')
  })
})
