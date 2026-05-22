import { useApp } from '../../context/useApp';
import './Navbar.css';

export default function Navbar() {
  const { lang, setLang, t, toggleTheme, theme } = useApp();

  const handleLang = () => setLang(lang === 'zh' ? 'en' : 'zh');

  const isDark = theme === 'dark' || (theme === 'auto' && window.matchMedia('(prefers-color-scheme:dark)').matches);

  const themeIcon = isDark
    ? <path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z"/>
    : <><circle cx="12" cy="12" r="4"/><path d="M12 2v2M12 20v2M4.93 4.93l1.41 1.41M17.66 17.66l1.41 1.41M2 12h2M20 12h2M6.34 17.66l-1.41 1.41M19.07 4.93l-1.41 1.41"/></>;

  const handleHamburger = () => {
    document.getElementById('hamburger')?.classList.toggle('open');
    document.getElementById('mobileMenu')?.classList.toggle('open');
  };

  const closeMobile = () => {
    document.getElementById('hamburger')?.classList.remove('open');
    document.getElementById('mobileMenu')?.classList.remove('open');
  };

  return (
    <>
      <nav className="nav" id="nav">
        <div className="nav-inner">
          <a href="#" className="nav-logo">
            <img src={`${import.meta.env.BASE_URL}logo.svg`} alt="DeviceDeck" className="nav-logo-icon" />
            <span>DeviceDeck</span>
          </a>
          <div className="nav-links" id="navLinks">
            <a href="#features">{t('nav.features')}</a>
            <a href="#screenshots">{t('nav.screenshots')}</a>
            <a href="#architecture">{t('nav.architecture')}</a>
            <a href="#download">{t('nav.download')}</a>
            <a href="#faq">{t('nav.faq')}</a>
          </div>
          <div className="nav-right">
            <a href="https://github.com/shenjianZ/DeviceDeck" className="gh-btn" title="GitHub" aria-label="GitHub" target="_blank" rel="noopener">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><path d="M15 22v-4a4.8 4.8 0 0 0-1-3.5c3 0 6-2 6-5.5.08-1.25-.27-2.48-1-3.5.28-1.15.28-2.35 0-3.5 0 0-1 0-3 1.5-2.64-.5-5.36-.5-8 0C6 2 5 2 5 2c-.3 1.15-.3 2.35 0 3.5A5.403 5.403 0 0 0 4 9c0 3.5 3 5.5 6 5.5-.39.49-.68 1.05-.85 1.65-.17.6-.22 1.23-.15 1.85v4"/><path d="M9 18c-4.51 2-5-2-7-2"/></svg>
            </a>
            <button className="lang-btn" onClick={handleLang} title="切换语言" aria-label="切换语言">
              {lang === 'zh' ? 'EN' : '中'}
            </button>
            <button className="theme-btn" onClick={toggleTheme} title="切换主题" aria-label="切换主题">
              <svg id="themeIcon" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">{themeIcon}</svg>
            </button>
            <button className="hamburger" id="hamburger" onClick={handleHamburger} aria-label="菜单">
              <span></span><span></span><span></span>
            </button>
          </div>
        </div>
      </nav>
      <div className="mobile-menu" id="mobileMenu">
        <a href="#features" onClick={closeMobile}>{t('nav.features')}</a>
        <a href="#screenshots" onClick={closeMobile}>{t('nav.screenshots')}</a>
        <a href="#architecture" onClick={closeMobile}>{t('nav.architecture')}</a>
        <a href="#download" onClick={closeMobile}>{t('nav.download')}</a>
        <a href="#faq" onClick={closeMobile}>{t('nav.faq')}</a>
      </div>
    </>
  );
}
