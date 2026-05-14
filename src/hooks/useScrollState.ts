import { useEffect, useRef, useState, useCallback } from "react";

interface ScrollState {
  /** 是否有顶部可滚动内容 */
  hasTop: boolean;
  /** 是否有底部可滚动内容 */
  hasBottom: boolean;
  /** 当前滚动位置 (0-1) */
  scrollRatio: number;
  /** 是否正在滚动 */
  isScrolling: boolean;
}

interface UseScrollStateOptions {
  /** 阈值，小于此值认为没有滚动 */
  threshold?: number;
  /** 滚动停止延迟 (ms) */
  scrollingDelay?: number;
}

/**
 * 检测滚动容器状态的 Hook
 * 用于实现滚动遮罩、滚动指示器等效果
 */
export function useScrollState(options: UseScrollStateOptions = {}) {
  const { threshold = 10, scrollingDelay = 150 } = options;
  const ref = useRef<HTMLDivElement>(null);
  const [state, setState] = useState<ScrollState>({
    hasTop: false,
    hasBottom: false,
    scrollRatio: 0,
    isScrolling: false,
  });

  const scrollingTimerRef = useRef<ReturnType<typeof setTimeout>>(undefined);

  const checkScroll = useCallback(() => {
    const el = ref.current;
    if (!el) return;

    const { scrollTop, scrollHeight, clientHeight } = el;
    const hasTop = scrollTop > threshold;
    const hasBottom = scrollTop < scrollHeight - clientHeight - threshold;
    const scrollRatio = scrollHeight > clientHeight
      ? scrollTop / (scrollHeight - clientHeight)
      : 0;

    setState((prev) => ({
      ...prev,
      hasTop,
      hasBottom,
      scrollRatio,
      isScrolling: true,
    }));

    // 清除之前的定时器
    if (scrollingTimerRef.current) {
      clearTimeout(scrollingTimerRef.current);
    }

    // 设置新的定时器，滚动停止后更新状态
    scrollingTimerRef.current = setTimeout(() => {
      setState((prev) => ({
        ...prev,
        isScrolling: false,
      }));
    }, scrollingDelay);
  }, [threshold, scrollingDelay]);

  useEffect(() => {
    const el = ref.current;
    if (!el) return;

    // 初始检查
    checkScroll();

    // 监听滚动事件
    el.addEventListener("scroll", checkScroll, { passive: true });

    // 监听内容变化
    const observer = new ResizeObserver(checkScroll);
    observer.observe(el);

    return () => {
      el.removeEventListener("scroll", checkScroll);
      observer.disconnect();
      if (scrollingTimerRef.current) {
        clearTimeout(scrollingTimerRef.current);
      }
    };
  }, [checkScroll]);

  return { ref, ...state };
}

/**
 * 滚动到顶部
 */
export function useScrollToTop() {
  const ref = useRef<HTMLDivElement>(null);

  const scrollToTop = useCallback((behavior: ScrollBehavior = "smooth") => {
    ref.current?.scrollTo({ top: 0, behavior });
  }, []);

  return { ref, scrollToTop };
}

/**
 * 滚动到底部
 */
export function useScrollToBottom() {
  const ref = useRef<HTMLDivElement>(null);

  const scrollToBottom = useCallback((behavior: ScrollBehavior = "smooth") => {
    const el = ref.current;
    if (el) {
      el.scrollTo({ top: el.scrollHeight, behavior });
    }
  }, []);

  return { ref, scrollToBottom };
}
