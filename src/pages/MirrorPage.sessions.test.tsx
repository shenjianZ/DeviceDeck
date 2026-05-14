import { fireEvent, render, screen, within } from '@testing-library/react'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import '../i18n'
import { useDeviceStore } from '../stores/deviceStore'
import { useMirrorStore } from '../stores/mirrorStore'
import { MirrorPage } from './MirrorPage'
import type { MirrorSession } from '../types'

const baseConfig = {
  maxSize: '1080',
  videoBitRate: '8M',
  maxFps: '60',
  videoCodec: 'h264',
  noControl: false,
  stayAwake: true,
  turnScreenOff: false,
}

function session(id: string, status: MirrorSession['status'], startedAt: number): MirrorSession {
  return {
    id,
    deviceSerial: id,
    platform: 'android',
    processId: null,
    status,
    startedAt,
    stoppedAt: status === 'running' ? null : startedAt + 1000,
    config: baseConfig,
  }
}

describe('MirrorPage sessions', () => {
  beforeEach(() => {
    useDeviceStore.setState({
      devices: [],
      wirelessServices: [],
      isScanning: false,
      isDiscoveringWireless: false,
      isWirelessBusy: false,
    })
    useMirrorStore.setState({
      sessions: [
        session('stopped-newest', 'stopped', 6000),
        session('running-oldest', 'running', 1000),
        session('failed-session', 'failed', 5000),
        session('stopped-middle', 'stopped', 4000),
        session('running-newest', 'running', 3000),
        session('stopped-oldest', 'stopped', 2000),
      ],
      isStarting: false,
      isStopping: null,
      stopMirror: vi.fn(),
    })
  })

  it('shows running sessions first and paginates the session history', () => {
    render(<MirrorPage />)

    const rows = screen.getAllByTestId('mirror-session-row')
    expect(rows).toHaveLength(5)
    expect(within(rows[0]).getByText('running-newest')).toBeInTheDocument()
    expect(within(rows[1]).getByText('running-oldest')).toBeInTheDocument()
    expect(screen.queryByText('stopped-oldest')).not.toBeInTheDocument()

    fireEvent.click(screen.getByRole('button', { name: '2' }))

    expect(screen.getByText('stopped-oldest')).toBeInTheDocument()
    expect(screen.queryByText('running-newest')).not.toBeInTheDocument()
  })
})
