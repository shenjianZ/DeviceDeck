import { useApp } from '../../context/useApp';
import './Architecture.css';

export default function Architecture() {
  const { t } = useApp();

  const layers = [
    {
      title: 'React + TypeScript',
      descKey: 'arch.l1',
      badges: ['React 19', 'TypeScript', 'TailwindCSS 4', 'Zustand', 'Vite 7'],
    },
    {
      title: 'Rust Core',
      descKey: 'arch.l2',
      badges: ['Tokio', 'rusqlite', 'Axum', 'Scrcpy', 'ADB'],
    },
    {
      title: 'Tauri 2',
      descKey: 'arch.l3',
      badges: ['Shell', 'Dialog', 'Updater', 'Autostart'],
    },
  ];

  return (
    <section className="section" id="architecture">
      <div className="container">
        <div className="section-header anim">
          <div className="section-tag">⬡ {t('arch.tag')}</div>
          <h2 className="section-title">{t('arch.title')}</h2>
          <p className="section-desc">{t('arch.desc')}</p>
        </div>
        <div className="arch-wrapper">
          <div className="arch-stack">
            {layers.map((layer, i) => (
              <div key={layer.title}>
                {i > 0 && (
                  <div className="arch-connector anim" style={{ '--d': `${i * 0.15 - 0.05}s` } as React.CSSProperties}>
                    <svg viewBox="0 0 40 40"><line className="line" x1="20" y1="4" x2="20" y2="30"/><polygon className="arrow" points="15,28 20,36 25,28"/></svg>
                  </div>
                )}
                <div className="arch-layer anim" style={{ '--d': `${i * 0.15}s` } as React.CSSProperties}>
                  <div className="layer-title">{layer.title}</div>
                  <div className="layer-desc">{t(layer.descKey)}</div>
                  <div className="arch-badges">
                    {layer.badges.map(b => <span className="arch-badge" key={b}>{b}</span>)}
                  </div>
                </div>
              </div>
            ))}
          </div>
        </div>
      </div>
    </section>
  );
}
