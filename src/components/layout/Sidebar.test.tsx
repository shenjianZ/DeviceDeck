import { render, screen } from '@testing-library/react'
import { describe, expect, it, vi } from 'vitest'
import '../../i18n'
import { Sidebar } from './Sidebar'

describe('Sidebar', () => {
  it('renders persistent text labels next to each navigation icon', () => {
    render(<Sidebar page="dashboard" onNav={vi.fn()} />)

    const labels = ['仪表盘', '设备', '投屏', '日志', '设置']

    for (const label of labels) {
      const button = screen.getByRole('button', { name: label })
      const text = button.querySelector('.sb-label')

      expect(text).toHaveTextContent(label)
      expect(button.querySelector('.sb-tip')).toBeNull()
    }
  })
})
