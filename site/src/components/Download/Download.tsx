import { useState } from 'react';
import { useApp } from '../../context/useApp';
import './Download.css';

type PlatformKey = 'win' | 'mac' | 'linux' | 'unknown';

type NavigatorWithUserAgentData = Navigator & {
  userAgentData?: {
    platform?: string;
  };
};

const platforms = [
  {
    id: 'dl-win',
    name: 'Windows',
    subtitle: 'x64 · Windows 10+',
    href: 'https://github.com/shenjianZ/DeviceDeck/releases/download/v0.1.1/DeviceDeck-v0.1.1-windows-x64.exe',
    releasesHref: 'https://github.com/shenjianZ/DeviceDeck/releases/tag/v0.1.1',
    osKey: 'win',
    icon: (
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round">
        <path d="M3 5.5L10.5 4.5V11.5H3V5.5Z"/><path d="M10.5 4.5L21 3V11.5H10.5V4.5Z"/>
        <path d="M3 11.5H10.5V18.5L3 17.5V11.5Z"/><path d="M10.5 11.5H21V20L10.5 18.5V11.5Z"/>
      </svg>
    ),
  },
  {
    id: 'dl-mac',
    name: 'macOS',
    subtitle: 'Apple Silicon & Intel · macOS 11+',
    href: 'https://github.com/shenjianZ/DeviceDeck/releases/download/v0.1.1/DeviceDeck-v0.1.1-macos-aarch64.dmg',
    releasesHref: 'https://github.com/shenjianZ/DeviceDeck/releases/tag/v0.1.1',
    osKey: 'mac',
    icon: (
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round">
        <path d="M18.71 19.5C17.88 20.74 17 21.95 15.66 21.97C14.32 21.99 13.89 21.18 12.37 21.18C10.84 21.18 10.37 21.95 9.07 21.99C7.79 20.04 6.25 18.09 5 16C3.39 13.29 3.08 10.55 4.22 8.73C5.35 6.93 7.2 6.04 8.93 6.04C10.25 6.04 11.32 6.84 12.17 6.84C13 6.84 14.23 5.92 15.79 6.02C16.54 6.04 18.19 6.26 19.32 7.58C19.27 7.63 17.2 8.87 17.23 11.36C17.26 14.34 19.86 15.36 19.89 15.37C19.86 15.44 19.42 16.91 18.39 18.41L18.71 19.5Z"/>
        <path d="M13 3.5C13.73 2.67 14.94 2.04 15.94 2C16.07 3.17 15.6 4.35 14.9 5.19C14.21 6.04 13.07 6.7 11.95 6.61C11.8 5.46 12.36 4.26 13 3.5Z"/>
      </svg>
    ),
  },
  {
    id: 'dl-linux',
    name: 'Linux',
    subtitle: 'x64 · Ubuntu 20.04+',
    href: 'https://github.com/shenjianZ/DeviceDeck/releases/download/v0.1.1/DeviceDeck-v0.1.1-linux-x64.AppImage',
    releasesHref: 'https://github.com/shenjianZ/DeviceDeck/releases/tag/v0.1.1',
    osKey: 'linux',
    icon: (
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round">
        <polyline points="4 17 10 11 4 5"/><line x1="12" y1="19" x2="20" y2="19"/>
      </svg>
    ),
  },
];

function detectPlatform(): PlatformKey {
  const nav = navigator as NavigatorWithUserAgentData;
  const uaDataPlatform = nav.userAgentData?.platform ?? '';
  const browserPlatform = navigator.platform ?? '';
  const userAgent = navigator.userAgent ?? '';
  const source = `${uaDataPlatform} ${browserPlatform} ${userAgent}`;

  if (/Windows|Win32|Win64|WOW64/i.test(source)) return 'win';
  if (/Macintosh|MacIntel|MacPPC|Mac68K|macOS|Mac OS/i.test(source)) return 'mac';
  if (/Linux|X11/i.test(source) && !/Android/i.test(source)) return 'linux';

  return 'unknown';
}

export default function Download() {
  const { t } = useApp();
  const [detectedPlatform] = useState<PlatformKey>(() => detectPlatform());

  return (
    <section className="section" id="download">
      <div className="container">
        <div className="section-header anim">
          <div className="section-tag">↓ {t('download.tag')}</div>
          <h2 className="section-title">{t('download.title')}</h2>
          <p className="section-desc">{t('download.desc')}</p>
        </div>
        <div className="download-grid">
          {platforms.map((p) => {
            const isDetected = detectedPlatform === p.osKey;
            return (
              <div
                className={`download-card anim${isDetected ? ' detected' : ''}`}
                id={p.id}
                key={p.id}
                style={{ '--d': '0s' } as React.CSSProperties}
                data-current={isDetected ? 'true' : undefined}
              >
                {isDetected && <span className="download-badge">{t('download.current')}</span>}
                <div className="download-icon">{p.icon}</div>
                <h3>{p.name}</h3>
                <div className="download-subtitle">{p.subtitle}</div>
                <a className="download-main-btn" href={p.href} target="_blank" rel="noopener">
                  {t('download.btn')}
                </a>
                <a className="download-other" href={p.releasesHref} target="_blank" rel="noopener">
                  {t('download.all')}
                  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><path d="M7 17L17 7"/><polyline points="7 7 17 7 17 17"/></svg>
                </a>
              </div>
            );
          })}
        </div>
      </div>
    </section>
  );
}
