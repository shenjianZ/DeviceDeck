import { useState, useEffect, useRef, useCallback } from 'react';
import { useApp } from '../../context/useApp';
import './Screenshots.css';

const slides = [
  {
    tab: '设备列表', tabEn: 'Device List',
    img: `${import.meta.env.BASE_URL}app-img/dashboard.png`,
    desc: { zh: '已连接设备总览，支持 USB / WiFi / 无线配对多种连接方式', en: 'Connected device overview — USB, WiFi, and wireless pairing supported' },
  },
  {
    tab: '屏幕镜像', tabEn: 'Screen Mirror',
    img: `${import.meta.env.BASE_URL}app-img/mirror.png`,
    desc: { zh: '基于 Scrcpy 的实时高清镜像，支持 H.265 编码与 60fps', en: 'Scrcpy-powered real-time HD mirroring with H.265 and 60fps' },
  },
  {
    tab: '设备详情', tabEn: 'Device Details',
    img: `${import.meta.env.BASE_URL}app-img/device.png`,
    desc: { zh: '设备信息、电池状态、存储容量一目了然', en: 'Device info, battery status, and storage at a glance' },
  },
  {
    tab: '系统日志', tabEn: 'System Logs',
    img: `${import.meta.env.BASE_URL}app-img/log.png`,
    desc: { zh: 'System / ADB / Scrcpy 三路日志实时聚合', en: 'Real-time aggregated System / ADB / Scrcpy logs' },
  },
  {
    tab: 'USB 传输', tabEn: 'USB Transfer',
    img: `${import.meta.env.BASE_URL}app-img/transfer-usb.png`,
    desc: { zh: '通过 USB 有线传输文件，稳定高速', en: 'Wired USB file transfer — stable and fast' },
  },
  {
    tab: 'WiFi 传输', tabEn: 'WiFi Transfer',
    img: `${import.meta.env.BASE_URL}app-img/transfer-wifi.png`,
    desc: { zh: 'WiFi 无线传输，无需数据线', en: 'Wireless WiFi transfer — no cables needed' },
  },
  {
    tab: 'WiFi Web 传输', tabEn: 'WiFi Web Transfer',
    img: `${import.meta.env.BASE_URL}app-img/wifi-transfer-web.png`,
    desc: { zh: '手机浏览器扫码即传，无需安装客户端', en: 'Scan QR to transfer from mobile browser — no app needed' },
  },
  {
    tab: '镜像设置', tabEn: 'Mirror Settings',
    img: `${import.meta.env.BASE_URL}app-img/settings-mirror.png`,
    desc: { zh: '分辨率、帧率、编码器、码率精细调控', en: 'Fine-tune resolution, FPS, codec, and bitrate' },
  },
  {
    tab: '工具设置', tabEn: 'Tools Settings',
    img: `${import.meta.env.BASE_URL}app-img/settings-tools.png`,
    desc: { zh: 'ADB 与 Scrcpy 工具路径自动检测与管理', en: 'Auto-detect and manage ADB & Scrcpy tool paths' },
  },
  {
    tab: '外观设置', tabEn: 'Appearance',
    img: `${import.meta.env.BASE_URL}app-img/settings-appearance.png`,
    desc: { zh: '深色 / 浅色主题切换，个性化界面风格', en: 'Dark / Light theme toggle for a personalized look' },
  },
  {
    tab: '关于', tabEn: 'About',
    img: `${import.meta.env.BASE_URL}app-img/settings-about.png`,
    desc: { zh: '版本信息与更新检查', en: 'Version info and update check' },
  },
];

export default function Screenshots() {
  const { lang, t } = useApp();
  const [idx, setIdx] = useState(0);
  const timerRef = useRef<ReturnType<typeof setInterval> | undefined>(undefined);

  const goSlide = useCallback((i: number) => {
    setIdx(((i % slides.length) + slides.length) % slides.length);
  }, []);

  const startAuto = useCallback(() => {
    clearInterval(timerRef.current);
    timerRef.current = setInterval(() => goSlide(idx + 1), 5000);
  }, [idx, goSlide]);

  useEffect(() => { startAuto(); return () => clearInterval(timerRef.current); }, [startAuto]);

  const handleNav = (i: number) => { goSlide(i); startAuto(); };

  return (
    <section className="section" id="screenshots">
      <div className="container">
        <div className="section-header anim">
          <div className="section-tag">◎ {t('screenshots.tag')}</div>
          <h2 className="section-title">{t('screenshots.title')}</h2>
          <p className="section-desc">{t('screenshots.desc')}</p>
        </div>
        <div className="carousel-container anim">
          <div className="carousel-viewport" onMouseEnter={() => clearInterval(timerRef.current)} onMouseLeave={startAuto}>
            <div className="carousel-track" style={{ transform: `translateX(-${idx * 100}%)` }}>
              {slides.map((slide, si) => (
                <div className="carousel-slide" key={si}>
                  <div className="slide-content">
                    <div className="slide-titlebar">
                      <span className="slide-dot r"></span><span className="slide-dot y"></span><span className="slide-dot g"></span>
                      <span className="slide-tab">{lang === 'zh' ? slide.tab : slide.tabEn}</span>
                    </div>
                    <div className="slide-screenshot">
                      <img src={slide.img} alt={lang === 'zh' ? slide.tab : slide.tabEn} loading="lazy" />
                    </div>
                    <div className="slide-caption">{lang === 'zh' ? slide.desc.zh : slide.desc.en}</div>
                  </div>
                </div>
              ))}
            </div>
          </div>
          <div className="slide-controls">
            <div className="slide-arrows">
              <button onClick={() => { goSlide(idx - 1); startAuto(); }} aria-label="上一张">
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><polyline points="15 18 9 12 15 6"/></svg>
              </button>
            </div>
            <div className="slide-dots">
              {slides.map((_, i) => (
                <button key={i} className={`slide-dot-btn${i === idx ? ' active' : ''}`} onClick={() => handleNav(i)}></button>
              ))}
            </div>
            <div className="slide-arrows">
              <button onClick={() => { goSlide(idx + 1); startAuto(); }} aria-label="下一张">
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><polyline points="9 18 15 12 9 6"/></svg>
              </button>
            </div>
          </div>
        </div>
      </div>
    </section>
  );
}
