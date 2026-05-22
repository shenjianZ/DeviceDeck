import { useEffect } from 'react';
import { useApp } from './context/useApp';
import { useAnimReveal } from './hooks/useIntersectionObserver';
import Navbar from './components/Navbar/Navbar';
import Hero from './components/Hero/Hero';
import Features from './components/Features/Features';
import Screenshots from './components/Screenshots/Screenshots';
import Architecture from './components/Architecture/Architecture';
import Download from './components/Download/Download';
import FAQ from './components/FAQ/FAQ';
import Footer from './components/Footer/Footer';

export default function App() {
  const { lang, theme } = useApp();

  // Apply theme to document
  useEffect(() => {
    document.documentElement.dataset.theme = theme;
  }, [theme]);

  // Update document language
  useEffect(() => {
    document.documentElement.lang = lang === 'zh' ? 'zh-CN' : 'en';
    document.title = lang === 'zh'
      ? 'DeviceDeck — Android 设备管理工作台'
      : 'DeviceDeck — Android Device Management Workbench';
  }, [lang]);

  // Hero reveal animation
  useEffect(() => {
    document.querySelectorAll('.hero .anim').forEach((el, i) => {
      const htmlEl = el as HTMLElement;
      htmlEl.style.opacity = '0';
      htmlEl.style.transform = 'translateY(32px)';
      setTimeout(() => {
        htmlEl.style.transition = 'opacity .7s cubic-bezier(.4,0,.2,1),transform .7s cubic-bezier(.4,0,.2,1)';
        htmlEl.style.opacity = '1';
        htmlEl.style.transform = 'translateY(0)';
      }, 120 + i * 140);
    });
  }, []);

  // Scroll animation observer
  useAnimReveal();

  // Nav shadow on scroll
  useEffect(() => {
    const handler = () => {
      const nav = document.getElementById('nav');
      if (nav) nav.style.boxShadow = window.scrollY > 20 ? 'var(--shadow)' : 'none';
    };
    window.addEventListener('scroll', handler, { passive: true });
    return () => window.removeEventListener('scroll', handler);
  }, []);

  return (
    <>
      <Navbar />
      <Hero />
      <Features />
      <Screenshots />
      <Architecture />
      <Download />
      <FAQ />
      <Footer />
    </>
  );
}
