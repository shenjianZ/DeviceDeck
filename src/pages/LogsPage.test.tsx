import { fireEvent, render, screen, within } from '@testing-library/react'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import '../i18n'
import { useLogStore } from '../stores/logStore'
import { LogsPage } from './LogsPage'
import { tauriApi } from '../lib/tauri'

vi.mock('../lib/tauri', () => ({
  tauriApi: {
    getLogsPaginated: vi.fn(),
    clearLogs: vi.fn(),
    onLog: vi.fn(async () => vi.fn()),
  },
}))

describe('LogsPage', () => {
  beforeEach(() => {
    vi.mocked(tauriApi.getLogsPaginated).mockResolvedValue({
      logs: [
        {
          id: 'log-1',
          time: new Date('2026-05-14T07:32:56Z').getTime(),
          source: 'scrcpy',
          level: 'error',
          deviceSerial: '192.168.43.187:5555',
          message: 'ERROR: Could not open icon image: D:\\rustproject\\DeviceDeck\\src-tauri\\icons\\icon.png',
        },
      ],
      total: 1,
      page: 1,
      page_size: 50,
      total_pages: 1,
    })
    useLogStore.setState({
      logs: [
        {
          id: 'log-1',
          time: new Date('2026-05-14T07:32:56Z').getTime(),
          source: 'scrcpy',
          level: 'error',
          deviceSerial: '192.168.43.187:5555',
          message: 'ERROR: Could not open icon image: D:\\rustproject\\DeviceDeck\\src-tauri\\icons\\icon.png',
        },
      ],
      total: 1,
      page: 1,
      pageSize: 50,
      totalPages: 1,
      isLoading: false,
      sourceFilter: 'all',
      levelFilter: 'all',
      isListening: false,
    })
  })

  it('expands a log row to show the full message', () => {
    render(<LogsPage />)

    expect(screen.queryByText('日志详情')).not.toBeInTheDocument()

    fireEvent.click(screen.getByRole('button', { name: /ERROR: Could not open icon image/ }))

    expect(screen.getByText('日志详情')).toBeInTheDocument()
    const detail = screen.getByTestId('log-detail-log-1')
    expect(within(detail).getByText('ERROR: Could not open icon image: D:\\rustproject\\DeviceDeck\\src-tauri\\icons\\icon.png')).toBeInTheDocument()
    expect(within(detail).getByText('192.168.43.187:5555')).toBeInTheDocument()
  })
})
