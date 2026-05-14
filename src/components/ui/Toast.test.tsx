import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest'
import { render, screen, fireEvent, act } from '@testing-library/react'
import { Toast } from './Toast'
import { useNotificationStore } from '../../stores/notificationStore'

describe('Toast 组件', () => {
  beforeEach(() => {
    useNotificationStore.setState({ notifications: [] })
    vi.useFakeTimers()
  })

  afterEach(() => {
    vi.useRealTimers()
  })

  it('没有通知时不渲染任何内容', () => {
    const { container } = render(<Toast />)
    expect(container.firstChild).toBeNull()
  })

  it('应该渲染成功通知', () => {
    useNotificationStore.getState().showSuccess('操作成功')

    render(<Toast />)

    expect(screen.getByText('操作成功')).toBeInTheDocument()
    expect(screen.getByTestId('toast')).toHaveAttribute('data-type', 'success')
  })

  it('应该渲染错误通知', () => {
    useNotificationStore.getState().showError('操作失败')

    render(<Toast />)

    expect(screen.getByText('操作失败')).toBeInTheDocument()
    expect(screen.getByTestId('toast')).toHaveAttribute('data-type', 'error')
  })

  it('应该渲染警告通知', () => {
    useNotificationStore.getState().showWarning('注意')

    render(<Toast />)

    expect(screen.getByText('注意')).toBeInTheDocument()
    expect(screen.getByTestId('toast')).toHaveAttribute('data-type', 'warning')
  })

  it('应该渲染信息通知', () => {
    useNotificationStore.getState().showInfo('提示')

    render(<Toast />)

    expect(screen.getByText('提示')).toBeInTheDocument()
    expect(screen.getByTestId('toast')).toHaveAttribute('data-type', 'info')
  })

  it('应该显示详细信息', () => {
    useNotificationStore.getState().showError('失败', '网络超时')

    render(<Toast />)

    expect(screen.getByText('网络超时')).toBeInTheDocument()
  })

  it('应该显示建议信息', () => {
    useNotificationStore.getState().addNotification({
      type: 'error',
      message: '失败',
      suggestion: '请重试',
    })

    render(<Toast />)

    expect(screen.getByText('请重试')).toBeInTheDocument()
  })

  it('点击关闭按钮应该移除通知', async () => {
    useNotificationStore.getState().showSuccess('临时消息')

    render(<Toast />)

    const closeButton = screen.getByRole('button', { name: /关闭/i })

    await act(async () => {
      fireEvent.click(closeButton)
      vi.advanceTimersByTime(300)
    })

    expect(useNotificationStore.getState().notifications).toHaveLength(0)
  })

  it('应该支持自动消失', async () => {
    useNotificationStore.getState().addNotification({
      type: 'success',
      message: '自动消失',
      duration: 3000,
    })

    render(<Toast />)

    expect(screen.getByText('自动消失')).toBeInTheDocument()

    await act(async () => {
      vi.advanceTimersByTime(3000)
      // 等待退出动画
      vi.advanceTimersByTime(300)
    })

    expect(useNotificationStore.getState().notifications).toHaveLength(0)
  })

  it('应该渲染多条通知', () => {
    const { showSuccess, showError } = useNotificationStore.getState()
    showSuccess('成功1')
    showError('失败1')

    render(<Toast />)

    expect(screen.getByText('成功1')).toBeInTheDocument()
    expect(screen.getByText('失败1')).toBeInTheDocument()
  })

  it('默认持续时间应该是 5000ms', async () => {
    useNotificationStore.getState().showSuccess('测试')

    render(<Toast />)

    expect(screen.getByText('测试')).toBeInTheDocument()

    await act(async () => {
      vi.advanceTimersByTime(4999)
    })
    expect(screen.getByText('测试')).toBeInTheDocument()

    await act(async () => {
      vi.advanceTimersByTime(1)
      vi.advanceTimersByTime(300)
    })
    expect(useNotificationStore.getState().notifications).toHaveLength(0)
  })

  it('duration 为 0 时不自动消失', async () => {
    useNotificationStore.getState().addNotification({
      type: 'info',
      message: '常驻消息',
      duration: 0,
    })

    render(<Toast />)

    await act(async () => {
      vi.advanceTimersByTime(10000)
    })

    expect(screen.getByText('常驻消息')).toBeInTheDocument()
  })
})
