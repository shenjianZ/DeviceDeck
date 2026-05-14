import { describe, expect, it } from 'vitest'
import css from '../index.css?raw'

function getRuleBody(selector: string) {
  const escapedSelector = selector.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')
  const match = css.match(new RegExp(`${escapedSelector}\\s*\\{([^}]*)\\}`))
  return match?.[1] ?? ''
}

describe('MirrorPage preset layout styles', () => {
  it('preset cards should fill their grid cell so right edges align with other cards', () => {
    const presetCard = getRuleBody('.preset-card')

    expect(presetCard).toContain('width: 100%;')
    expect(presetCard).not.toContain('max-width:')
  })
})
