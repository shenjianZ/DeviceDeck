import { useEffect, useRef } from 'react';

export function useAnimReveal() {
  const obsRef = useRef<IntersectionObserver | null>(null);

  useEffect(() => {
    obsRef.current = new IntersectionObserver(
      (entries) => {
        entries.forEach((e) => {
          if (e.isIntersecting) {
            const d = parseFloat(getComputedStyle(e.target).getPropertyValue('--d')) || 0;
            setTimeout(() => e.target.classList.add('vis'), d * 1000);
            obsRef.current?.unobserve(e.target);
          }
        });
      },
      { threshold: 0.1, rootMargin: '0px 0px -40px 0px' }
    );

    document.querySelectorAll('.anim:not(.hero .anim)').forEach((el) => {
      obsRef.current?.observe(el);
    });

    return () => obsRef.current?.disconnect();
  }, []);
}
