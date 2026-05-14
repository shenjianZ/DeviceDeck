import { render, screen } from '@testing-library/react'
import { describe, expect, it } from 'vitest'
import '../../i18n'
import { Topbar } from './Topbar'

describe('Topbar', () => {
  it('shows the app name without a page title', () => {
    render(
      <Topbar
        theme="light"
        onToggleTheme={() => undefined}
        environment={null}
      />
    )

    expect(screen.getByText('DeviceDeck')).toBeInTheDocument()
    expect(screen.queryByText('设置')).not.toBeInTheDocument()
    expect(document.querySelector('.topbar-title')).toBeNull()
  })
})
