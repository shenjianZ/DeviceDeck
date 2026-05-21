import { describe, expect, it, vi } from 'vitest'
import {
  blockContextMenu,
  blockShortcutKeyDown,
  installContextMenuBlocker,
  installShortcutBlocker,
  isShortcutKeyEvent,
} from './keyboardShortcuts'

function keyboardEvent(key: string, init: Partial<KeyboardEvent> = {}) {
  return new KeyboardEvent('keydown', {
    key,
    bubbles: true,
    cancelable: true,
    ...init,
  })
}

describe('keyboard shortcut blocking', () => {
  it('detects modifier and function-key shortcuts', () => {
    expect(isShortcutKeyEvent(keyboardEvent('r', { ctrlKey: true }))).toBe(true)
    expect(isShortcutKeyEvent(keyboardEvent('v', { metaKey: true }))).toBe(true)
    expect(isShortcutKeyEvent(keyboardEvent('ArrowLeft', { altKey: true }))).toBe(true)
    expect(isShortcutKeyEvent(keyboardEvent('F1'))).toBe(true)
    expect(isShortcutKeyEvent(keyboardEvent('F12'))).toBe(true)

    expect(isShortcutKeyEvent(keyboardEvent('a'))).toBe(false)
    expect(isShortcutKeyEvent(keyboardEvent('Backspace'))).toBe(false)
    expect(isShortcutKeyEvent(keyboardEvent('Tab'))).toBe(false)
  })

  it('prevents shortcut keydown events', () => {
    const shortcut = keyboardEvent('r', { ctrlKey: true })
    const normalKey = keyboardEvent('a')

    blockShortcutKeyDown(shortcut)
    blockShortcutKeyDown(normalKey)

    expect(shortcut.defaultPrevented).toBe(true)
    expect(normalKey.defaultPrevented).toBe(false)
  })

  it('allows select-all inside an app shortcut scope', () => {
    const scope = document.createElement('div')
    scope.dataset.ddAppShortcuts = 'file-transfer'
    document.body.appendChild(scope)

    const shortcut = keyboardEvent('a', { ctrlKey: true })

    blockShortcutKeyDown(shortcut)

    expect(shortcut.defaultPrevented).toBe(false)
    scope.remove()
  })

  it('allows shortcuts in editable targets', () => {
    const input = document.createElement('input')
    document.body.appendChild(input)
    const shortcut = keyboardEvent('a', { ctrlKey: true })
    Object.defineProperty(shortcut, 'target', { value: input })

    blockShortcutKeyDown(shortcut)

    expect(shortcut.defaultPrevented).toBe(false)
    input.remove()
  })

  it('installs a capture-phase document shortcut blocker', () => {
    const addEventListener = vi.fn()
    const removeEventListener = vi.fn()

    const uninstall = installShortcutBlocker({
      addEventListener,
      removeEventListener,
    })

    expect(addEventListener).toHaveBeenCalledWith('keydown', blockShortcutKeyDown, {
      capture: true,
    })

    uninstall()

    expect(removeEventListener).toHaveBeenCalledWith('keydown', blockShortcutKeyDown, {
      capture: true,
    })
  })

  it('installs context menu blocking only when enabled', () => {
    const addEventListener = vi.fn()
    const removeEventListener = vi.fn()

    const uninstallDev = installContextMenuBlocker({
      addEventListener,
      removeEventListener,
    }, false)

    expect(addEventListener).not.toHaveBeenCalled()
    uninstallDev()
    expect(removeEventListener).not.toHaveBeenCalled()

    const uninstallProd = installContextMenuBlocker({
      addEventListener,
      removeEventListener,
    }, true)

    expect(addEventListener).toHaveBeenCalledWith('contextmenu', blockContextMenu, {
      capture: true,
    })

    uninstallProd()

    expect(removeEventListener).toHaveBeenCalledWith('contextmenu', blockContextMenu, {
      capture: true,
    })
  })

  it('prevents context menu events', () => {
    const event = new MouseEvent('contextmenu', { bubbles: true, cancelable: true })

    blockContextMenu(event)

    expect(event.defaultPrevented).toBe(true)
  })

  it('allows context menus inside an app context-menu scope', () => {
    const scope = document.createElement('div')
    scope.dataset.ddContextMenu = 'file-transfer'
    document.body.appendChild(scope)
    const event = new MouseEvent('contextmenu', { bubbles: true, cancelable: true })
    Object.defineProperty(event, 'target', { value: scope })

    blockContextMenu(event)

    expect(event.defaultPrevented).toBe(false)
    scope.remove()
  })
})
