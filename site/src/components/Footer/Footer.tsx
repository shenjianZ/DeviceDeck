import { useApp } from '../../context/useApp';
import './Footer.css';

export default function Footer() {
  const { t } = useApp();

  return (
    <footer className="footer">
      <div className="container">
        <div className="footer-inner">
          <div className="footer-brand">
            <div className="logo"><img src={`${import.meta.env.BASE_URL}logo.svg`} alt="DeviceDeck" className="logo-icon" />DeviceDeck</div>
            <p>{t('footer.desc')}</p>
          </div>
          <div className="footer-col">
            <h4>{t('footer.resources')}</h4>
            <a href="https://github.com/shenjianZ/DeviceDeck" target="_blank" rel="noopener">GitHub</a>
            <a href="https://github.com/shenjianZ/DeviceDeck/releases" target="_blank" rel="noopener">{t('footer.releases')}</a>
            <a href="https://github.com/shenjianZ/DeviceDeck/issues" target="_blank" rel="noopener">{t('footer.issues')}</a>
            <a href="https://github.com/shenjianZ/DeviceDeck/blob/main/README.md" target="_blank" rel="noopener">{t('footer.docs')}</a>
          </div>
          <div className="footer-col">
            <h4>{t('footer.community')}</h4>
            <a href="https://github.com/shenjianZ/DeviceDeck/blob/main/CONTRIBUTING.md" target="_blank" rel="noopener">{t('footer.contribute')}</a>
            <a href="https://github.com/shenjianZ/DeviceDeck/discussions" target="_blank" rel="noopener">{t('footer.discussions')}</a>
          </div>
        </div>
        <div className="footer-bottom">
          <p>&copy; 2026 shenjianZ. {t('footer.license')}</p>
        </div>
      </div>
    </footer>
  );
}
