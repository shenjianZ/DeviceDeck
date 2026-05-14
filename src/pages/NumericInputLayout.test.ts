import { describe, expect, it } from 'vitest'
import mirrorPageSource from './MirrorPage.tsx?raw'
import settingsPageSource from './SettingsPage.tsx?raw'

function count(source: string, token: string) {
  return source.split(token).length - 1
}

describe('numeric text inputs', () => {
  it('uses text inputs with numeric filtering instead of browser number spinners', () => {
    const source = `${settingsPageSource}\n${mirrorPageSource}`

    expect(source).not.toContain('type="number"')
    expect(count(source, 'type="text"')).toBeGreaterThanOrEqual(4)
    expect(count(source, 'inputMode="numeric"')).toBe(4)
    expect(count(source, 'pattern="[0-9]*"')).toBe(4)
    expect(count(source, 'replace(/\\D/g, "")')).toBeGreaterThanOrEqual(4)
  })
})
