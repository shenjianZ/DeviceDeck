import { fireEvent, render, screen } from '@testing-library/react'
import { describe, expect, it } from 'vitest'
import '../i18n'
import { MirrorPage } from './MirrorPage'
import { SettingsPage } from './SettingsPage'

describe('mirror settings placement', () => {
  it('shows the full mirror parameter presets in the Settings mirror section', () => {
    render(<SettingsPage />)

    fireEvent.click(screen.getByRole('button', { name: '投屏' }))

    expect(screen.getByRole('heading', { name: '投屏参数' })).toBeInTheDocument()
    expect(screen.getByText('流畅模式')).toBeInTheDocument()
    expect(screen.getByText('高清模式')).toBeInTheDocument()
    expect(screen.getByText('极清模式')).toBeInTheDocument()
    expect(screen.getByText('H.265 极致')).toBeInTheDocument()
    expect(screen.getByText('只读模式')).toBeInTheDocument()
    expect(screen.getByText('保持唤醒')).toBeInTheDocument()
    expect(screen.getByText('关闭设备屏幕')).toBeInTheDocument()
  })

  it('removes mirror parameter presets from the Mirror page', () => {
    render(<MirrorPage />)

    expect(screen.queryByText('流畅模式')).not.toBeInTheDocument()
    expect(screen.queryByText('高清模式')).not.toBeInTheDocument()
    expect(screen.queryByText('极清模式')).not.toBeInTheDocument()
    expect(screen.queryByText('H.265 极致')).not.toBeInTheDocument()
  })
})
