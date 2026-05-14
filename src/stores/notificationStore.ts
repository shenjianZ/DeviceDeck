import { create } from 'zustand'

export type NotificationType = 'success' | 'error' | 'warning' | 'info'

export interface Notification {
  id: string
  type: NotificationType
  message: string
  detail?: string | null
  suggestion?: string | null
  duration?: number
}

interface NotificationStore {
  notifications: Notification[]

  addNotification: (notification: Omit<Notification, 'id'>) => void
  removeNotification: (id: string) => void
  clearAll: () => void

  showSuccess: (message: string, detail?: string | null) => void
  showError: (message: string, detail?: string | null, suggestion?: string | null) => void
  showWarning: (message: string, detail?: string | null) => void
  showInfo: (message: string, detail?: string | null) => void
}

let nextId = 0

function generateId(): string {
  return `notification-${Date.now()}-${nextId++}`
}

export const useNotificationStore = create<NotificationStore>((set) => ({
  notifications: [],

  addNotification: (notification) => {
    const id = generateId()
    set((state) => ({
      notifications: [...state.notifications, { ...notification, id }],
    }))
  },

  removeNotification: (id) => {
    set((state) => ({
      notifications: state.notifications.filter((n) => n.id !== id),
    }))
  },

  clearAll: () => {
    set({ notifications: [] })
  },

  showSuccess: (message, detail) => {
    const id = generateId()
    set((state) => ({
      notifications: [
        ...state.notifications,
        { id, type: 'success', message, detail },
      ],
    }))
  },

  showError: (message, detail, suggestion) => {
    const id = generateId()
    set((state) => ({
      notifications: [
        ...state.notifications,
        { id, type: 'error', message, detail, suggestion },
      ],
    }))
  },

  showWarning: (message, detail) => {
    const id = generateId()
    set((state) => ({
      notifications: [
        ...state.notifications,
        { id, type: 'warning', message, detail },
      ],
    }))
  },

  showInfo: (message, detail) => {
    const id = generateId()
    set((state) => ({
      notifications: [
        ...state.notifications,
        { id, type: 'info', message, detail },
      ],
    }))
  },
}))
