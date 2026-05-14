import { useEffect, useRef, useState, type JSX } from 'react'
import { useNotificationStore, type Notification, type NotificationType } from '../../stores/notificationStore'

/* ── 精致的 SVG 图标 ──────────────────────────────────────── */

const SuccessIcon = () => (
  <svg viewBox="0 0 24 24" fill="none" className="toast-icon">
    <circle cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="1.5" opacity="0.3" />
    <path
      d="M8 12.5L10.5 15L16 9.5"
      stroke="currentColor"
      strokeWidth="1.5"
      strokeLinecap="round"
      strokeLinejoin="round"
    />
  </svg>
)

const ErrorIcon = () => (
  <svg viewBox="0 0 24 24" fill="none" className="toast-icon">
    <circle cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="1.5" opacity="0.3" />
    <path
      d="M15 9L9 15M9 9L15 15"
      stroke="currentColor"
      strokeWidth="1.5"
      strokeLinecap="round"
    />
  </svg>
)

const WarningIcon = () => (
  <svg viewBox="0 0 24 24" fill="none" className="toast-icon">
    <path
      d="M12 3L22 20H2L12 3Z"
      stroke="currentColor"
      strokeWidth="1.5"
      strokeLinejoin="round"
      opacity="0.3"
    />
    <path
      d="M12 10V14"
      stroke="currentColor"
      strokeWidth="1.5"
      strokeLinecap="round"
    />
    <circle cx="12" cy="17" r="0.75" fill="currentColor" />
  </svg>
)

const InfoIcon = () => (
  <svg viewBox="0 0 24 24" fill="none" className="toast-icon">
    <circle cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="1.5" opacity="0.3" />
    <path
      d="M12 10V17"
      stroke="currentColor"
      strokeWidth="1.5"
      strokeLinecap="round"
    />
    <circle cx="12" cy="7" r="0.75" fill="currentColor" />
  </svg>
)

const iconMap: Record<NotificationType, () => JSX.Element> = {
  success: SuccessIcon,
  error: ErrorIcon,
  warning: WarningIcon,
  info: InfoIcon,
}

/* ── 进度条组件 ──────────────────────────────────────────── */

function ProgressBar({ duration, type }: { duration: number; type: NotificationType }) {
  const [progress, setProgress] = useState(100)
  const startTimeRef = useRef(Date.now())
  const rafRef = useRef<number>(0)

  useEffect(() => {
    if (duration === 0) return

    const animate = () => {
      const elapsed = Date.now() - startTimeRef.current
      const remaining = Math.max(0, 100 - (elapsed / duration) * 100)
      setProgress(remaining)

      if (remaining > 0) {
        rafRef.current = requestAnimationFrame(animate)
      }
    }

    rafRef.current = requestAnimationFrame(animate)

    return () => {
      if (rafRef.current) cancelAnimationFrame(rafRef.current)
    }
  }, [duration])

  if (duration === 0) return null

  return (
    <div className="toast-progress-track">
      <div
        className={`toast-progress-bar toast-progress-${type}`}
        style={{ width: `${progress}%` }}
      />
    </div>
  )
}

/* ── 关闭按钮 ────────────────────────────────────────────── */

function CloseButton({ onClick }: { onClick: () => void }) {
  return (
    <button
      onClick={onClick}
      className="toast-close"
      aria-label="关闭"
    >
      <svg viewBox="0 0 16 16" fill="none">
        <path
          d="M12 4L4 12M4 4L12 12"
          stroke="currentColor"
          strokeWidth="1.2"
          strokeLinecap="round"
        />
      </svg>
    </button>
  )
}

/* �─ Toast 单项 ───────────────────────────────────────────── */

function ToastItem({ notification }: { notification: Notification }) {
  const { removeNotification } = useNotificationStore()
  const [isVisible, setIsVisible] = useState(false)
  const Icon = iconMap[notification.type]

  useEffect(() => {
    // 触发进入动画
    requestAnimationFrame(() => setIsVisible(true))
  }, [])

  useEffect(() => {
    if (notification.duration === 0) return

    const timer = setTimeout(() => {
      setIsVisible(false)
      setTimeout(() => removeNotification(notification.id), 300)
    }, notification.duration || 5000)

    return () => clearTimeout(timer)
  }, [notification.id, notification.duration, removeNotification])

  const handleClose = () => {
    setIsVisible(false)
    setTimeout(() => removeNotification(notification.id), 300)
  }

  return (
    <div
      data-testid="toast"
      data-type={notification.type}
      className={`toast-item ${isVisible ? 'toast-visible' : 'toast-hidden'}`}
    >
      {/* 左侧光条 */}
      <div className={`toast-glow toast-glow-${notification.type}`} />

      {/* 主内容区 */}
      <div className="toast-content">
        {/* 图标 */}
        <div className={`toast-icon-wrap toast-icon-${notification.type}`}>
          <Icon />
        </div>

        {/* 文本 */}
        <div className="toast-text">
          <p className="toast-message">{notification.message}</p>
          {notification.detail && (
            <p className="toast-detail">{notification.detail}</p>
          )}
          {notification.suggestion && (
            <p className="toast-suggestion">{notification.suggestion}</p>
          )}
        </div>

        {/* 关闭按钮 */}
        <CloseButton onClick={handleClose} />
      </div>

      {/* 进度条 */}
      <ProgressBar
        duration={notification.duration || 5000}
        type={notification.type}
      />
    </div>
  )
}

/* ── Toast 容器 ──────────────────────────────────────────── */

export function Toast() {
  const { notifications } = useNotificationStore()

  if (notifications.length === 0) return null

  return (
    <div className="toast-container">
      {notifications.map((notification, index) => (
        <div
          key={notification.id}
          style={{ '--toast-index': index } as React.CSSProperties}
        >
          <ToastItem notification={notification} />
        </div>
      ))}
    </div>
  )
}
