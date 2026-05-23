import { useApp } from '../../context/useApp';
import './Hero.css';

export default function Hero() {
  const { lang, t } = useApp();

  return (
    <section className="hero" id="hero">
      <div className="hero-grid"></div>
      <div className="hero-glow g1"></div>
      <div className="hero-glow g2"></div>
      <div className="container hero-inner">
        <div className="hero-content">
          <div className="hero-badge anim">
            <span className="dot"></span>
            <span>{t('hero.badge')}</span>
          </div>
          <h1 className="hero-title anim">
            {lang === 'zh' ? (
              <span>Android 设备管理<br/>从此<span className="accent">得心应手</span></span>
            ) : (
              <span>Android Device<br/>Management <span className="accent">Redefined</span></span>
            )}
          </h1>
          <p className="hero-sub anim">{t('hero.subtitle')}</p>
          <div className="hero-actions anim">
            <a href="https://github.com/shenjianZ/DeviceDeck/releases" className="btn-primary" target="_blank" rel="noopener">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><polyline points="7 10 12 15 17 10"/><line x1="12" y1="15" x2="12" y2="3"/></svg>
              {lang === 'zh' ? '立即下载' : 'Download'}
            </a>
            <a href="https://github.com/shenjianZ/DeviceDeck" className="btn-secondary" target="_blank" rel="noopener">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><path d="M15 22v-4a4.8 4.8 0 0 0-1-3.5c3 0 6-2 6-5.5.08-1.25-.27-2.48-1-3.5.28-1.15.28-2.35 0-3.5 0 0-1 0-3 1.5-2.64-.5-5.36-.5-8 0C6 2 5 2 5 2c-.3 1.15-.3 2.35 0 3.5A5.403 5.403 0 0 0 4 9c0 3.5 3 5.5 6 5.5-.39.49-.68 1.05-.85 1.65-.17.6-.22 1.23-.15 1.85v4"/><path d="M9 18c-4.51 2-5-2-7-2"/></svg>
              {t('hero.source')}
            </a>
          </div>
          <div className="hero-platform anim">
            <span className="platform-badge accent">v0.1.3</span>
            <span className="platform-badge">Windows</span>
            <span className="platform-badge">macOS</span>
            <span className="platform-badge">Linux</span>
          </div>
        </div>
        <div className="hero-visual anim">
          <div className="app-mockup">
            <div className="mockup-titlebar">
              <span className="mockup-dot r"></span><span className="mockup-dot y"></span><span className="mockup-dot g"></span>
              <span className="mockup-tab">DeviceDeck</span>
            </div>
            <div className="mockup-body">
              <div className="mockup-sidebar">
                <h4>{lang === 'zh' ? '设备列表' : 'Device List'}</h4>
                <div className="mockup-device active"><span className="status on"></span><div><div className="name">Pixel 8 Pro</div><div className="info">Android 14 · USB</div></div></div>
                <div className="mockup-device"><span className="status on"></span><div><div className="name">Galaxy S24</div><div className="info">Android 14 · WiFi</div></div></div>
                <div className="mockup-device"><span className="status off"></span><div><div className="name">Redmi Note 13</div><div className="info">{lang === 'zh' ? 'Android 13 · 离线' : 'Android 13 · Offline'}</div></div></div>
              </div>
              <div className="mockup-main">
                <h4>{lang === 'zh' ? '屏幕镜像' : 'Screen Mirror'}</h4>
                <div className="mockup-screen"><div className="bar"></div></div>
                <div className="mockup-stats">
                  <div className="mockup-stat"><div className="label">{lang === 'zh' ? '编码' : 'Codec'}</div><div className="value">H.265</div></div>
                  <div className="mockup-stat"><div className="label">{lang === 'zh' ? '帧率' : 'FPS'}</div><div className="value">60 fps</div></div>
                  <div className="mockup-stat"><div className="label">{lang === 'zh' ? '码率' : 'Bitrate'}</div><div className="value">8 Mbps</div></div>
                  <div className="mockup-stat"><div className="label">{lang === 'zh' ? '延迟' : 'Latency'}</div><div className="value">~12 ms</div></div>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </section>
  );
}
