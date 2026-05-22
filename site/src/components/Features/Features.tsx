import { useApp } from '../../context/useApp';
import './Features.css';

const features = [
  { icon: '🔌', key: 'f1' },
  { icon: '🖥️', key: 'f2' },
  { icon: '📊', key: 'f3' },
  { icon: '⚙️', key: 'f4' },
  { icon: '📋', key: 'f5' },
  { icon: '🔍', key: 'f6' },
];

export default function Features() {
  const { t } = useApp();

  return (
    <section className="section" id="features">
      <div className="container">
        <div className="section-header anim">
          <div className="section-tag">✦ {t('features.tag')}</div>
          <h2 className="section-title">{t('features.title')}</h2>
          <p className="section-desc">{t('features.desc')}</p>
        </div>
        <div className="features-grid">
          {features.map((f, i) => (
            <div className="feature-card anim" key={f.key} style={{ '--d': `${i * 0.05}s` } as React.CSSProperties}>
              <div className="feature-icon">{f.icon}</div>
              <h3>{t(`${f.key}.t`)}</h3>
              <p>{t(`${f.key}.d`)}</p>
            </div>
          ))}
        </div>
      </div>
    </section>
  );
}
