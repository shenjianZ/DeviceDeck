import { useState } from 'react';
import { useApp } from '../../context/useApp';
import './FAQ.css';

const faqKeys = ['faq.q1', 'faq.q2', 'faq.q3', 'faq.q4', 'faq.q5', 'faq.q6'];

export default function FAQ() {
  const { t } = useApp();
  const [openIdx, setOpenIdx] = useState<number | null>(null);

  return (
    <section className="section" id="faq">
      <div className="container">
        <div className="section-header anim">
          <div className="section-tag">? {t('faq.tag')}</div>
          <h2 className="section-title">{t('faq.title')}</h2>
        </div>
        <div className="faq-list">
          {faqKeys.map((qKey, i) => {
            const aKey = qKey.replace('.q', '.a');
            const isOpen = openIdx === i;
            const panelId = `faq-panel-${i}`;
            return (
              <div className={`faq-item${isOpen ? ' open' : ''}`} key={qKey} style={{ '--d': `${i * 0.06}s` } as React.CSSProperties}>
                <button
                  className="faq-q"
                  type="button"
                  aria-expanded={isOpen}
                  aria-controls={panelId}
                  onClick={() => setOpenIdx(isOpen ? null : i)}
                >
                  {t(qKey)}
                  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><line x1="12" y1="5" x2="12" y2="19"/><line x1="5" y1="12" x2="19" y2="12"/></svg>
                </button>
                <div className="faq-a" id={panelId} aria-hidden={!isOpen}>
                  <div className="faq-a-content">{t(aKey)}</div>
                </div>
              </div>
            );
          })}
        </div>
      </div>
    </section>
  );
}
