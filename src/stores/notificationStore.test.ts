import { describe, it, expect, beforeEach } from 'vitest'
import { useNotificationStore } from './notificationStore'

describe('notificationStore', () => {
  beforeEach(() => {
    // 每个测试前重置 store
    useNotificationStore.setState({ notifications: [] })
  })

  it('初始状态应该为空数组', () => {
    const { notifications } = useNotificationStore.getState()
    expect(notifications).toEqual([])
  })

  it('addNotification 应该添加一条通知', () => {
    const { addNotification } = useNotificationStore.getState()

    addNotification({
      type: 'success',
      message: '操作成功',
    })

    const { notifications } = useNotificationStore.getState()
    expect(notifications).toHaveLength(1)
    expect(notifications[0].type).toBe('success')
    expect(notifications[0].message).toBe('操作成功')
    expect(notifications[0].id).toBeDefined()
  })

  it('addNotification 应该支持详细信息', () => {
    const { addNotification } = useNotificationStore.getState()

    addNotification({
      type: 'error',
      message: '操作失败',
      detail: '网络连接超时',
      suggestion: '请检查网络设置',
    })

    const { notifications } = useNotificationStore.getState()
    expect(notifications[0].detail).toBe('网络连接超时')
    expect(notifications[0].suggestion).toBe('请检查网络设置')
  })

  it('removeNotification 应该移除指定通知', () => {
    const { addNotification, removeNotification } = useNotificationStore.getState()

    addNotification({ type: 'success', message: '消息1' })
    addNotification({ type: 'error', message: '消息2' })

    const { notifications } = useNotificationStore.getState()
    expect(notifications).toHaveLength(2)

    removeNotification(notifications[0].id)

    const { notifications: remaining } = useNotificationStore.getState()
    expect(remaining).toHaveLength(1)
    expect(remaining[0].message).toBe('消息2')
  })

  it('clearAll 应该清除所有通知', () => {
    const { addNotification, clearAll } = useNotificationStore.getState()

    addNotification({ type: 'success', message: '消息1' })
    addNotification({ type: 'error', message: '消息2' })
    addNotification({ type: 'warning', message: '消息3' })

    expect(useNotificationStore.getState().notifications).toHaveLength(3)

    clearAll()

    expect(useNotificationStore.getState().notifications).toEqual([])
  })

  it('应该支持 success, error, warning, info 四种类型', () => {
    const { addNotification } = useNotificationStore.getState()

    addNotification({ type: 'success', message: '成功' })
    addNotification({ type: 'error', message: '错误' })
    addNotification({ type: 'warning', message: '警告' })
    addNotification({ type: 'info', message: '信息' })

    const { notifications } = useNotificationStore.getState()
    expect(notifications).toHaveLength(4)
    expect(notifications.map(n => n.type)).toEqual(['success', 'error', 'warning', 'info'])
  })

  it('每条通知应该有唯一的 id', () => {
    const { addNotification } = useNotificationStore.getState()

    addNotification({ type: 'success', message: '消息1' })
    addNotification({ type: 'success', message: '消息2' })

    const { notifications } = useNotificationStore.getState()
    expect(notifications[0].id).not.toBe(notifications[1].id)
  })

  it('应该支持设置持续时间', () => {
    const { addNotification } = useNotificationStore.getState()

    addNotification({
      type: 'success',
      message: '临时消息',
      duration: 5000,
    })

    const { notifications } = useNotificationStore.getState()
    expect(notifications[0].duration).toBe(5000)
  })

  it('应该支持便捷方法 showSuccess', () => {
    const { showSuccess } = useNotificationStore.getState()

    showSuccess('保存成功')

    const { notifications } = useNotificationStore.getState()
    expect(notifications).toHaveLength(1)
    expect(notifications[0].type).toBe('success')
    expect(notifications[0].message).toBe('保存成功')
  })

  it('应该支持便捷方法 showError', () => {
    const { showError } = useNotificationStore.getState()

    showError('操作失败', '网络错误')

    const { notifications } = useNotificationStore.getState()
    expect(notifications).toHaveLength(1)
    expect(notifications[0].type).toBe('error')
    expect(notifications[0].message).toBe('操作失败')
    expect(notifications[0].detail).toBe('网络错误')
  })

  it('应该支持便捷方法 showWarning', () => {
    const { showWarning } = useNotificationStore.getState()

    showWarning('注意')

    const { notifications } = useNotificationStore.getState()
    expect(notifications[0].type).toBe('warning')
  })

  it('应该支持便捷方法 showInfo', () => {
    const { showInfo } = useNotificationStore.getState()

    showInfo('提示信息')

    const { notifications } = useNotificationStore.getState()
    expect(notifications[0].type).toBe('info')
  })
})
