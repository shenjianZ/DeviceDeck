import { type ReactNode, type CSSProperties } from "react";
import { useScrollState } from "../../hooks/useScrollState";
import { ChevronUp } from "lucide-react";

interface ScrollFadeProps {
  children: ReactNode;
  /** 最大高度 */
  maxHeight?: string | number;
  /** 遮罩高度 */
  fadeHeight?: number;
  /** 自定义类名 */
  className?: string;
  /** 自定义样式 */
  style?: CSSProperties;
  /** 是否显示滚动到顶部按钮 */
  showScrollToTop?: boolean;
}

/**
 * 带渐变遮罩的滚动容器
 * 在内容溢出时自动显示顶部/底部渐变遮罩，提示用户可以滚动
 */
export function ScrollFade({
  children,
  maxHeight = "100%",
  fadeHeight = 32,
  className = "",
  style,
  showScrollToTop = false,
}: ScrollFadeProps) {
  const { ref, hasTop, hasBottom, scrollRatio } = useScrollState({ threshold: 10 });

  const scrollToTop = () => {
    ref.current?.scrollTo({ top: 0, behavior: "smooth" });
  };

  return (
    <div
      ref={ref}
      className={`optimize-scroll ${className}`}
      style={{
        maxHeight,
        overflowY: "auto",
        position: "relative",
        ...style,
      }}
    >
      {/* 顶部渐变遮罩 */}
      <div
        style={{
          position: "sticky",
          top: 0,
          left: 0,
          right: 0,
          height: fadeHeight,
          background: `linear-gradient(to bottom, var(--card) 0%, transparent 100%)`,
          zIndex: 5,
          pointerEvents: "none",
          transition: "opacity 0.3s ease",
          opacity: hasTop ? 1 : 0,
          marginTop: -fadeHeight,
          marginBottom: fadeHeight,
        }}
      />

      {children}

      {/* 底部渐变遮罩 */}
      <div
        style={{
          position: "sticky",
          bottom: 0,
          left: 0,
          right: 0,
          height: fadeHeight,
          background: `linear-gradient(to top, var(--card) 0%, transparent 100%)`,
          zIndex: 5,
          pointerEvents: "none",
          transition: "opacity 0.3s ease",
          opacity: hasBottom ? 1 : 0,
        }}
      />

      {/* 滚动到顶部按钮 */}
      {showScrollToTop && hasTop && (
        <button
          onClick={scrollToTop}
          className="scroll-to-top-btn"
          style={{
            position: "absolute",
            bottom: 16,
            right: 16,
            width: 36,
            height: 36,
            borderRadius: "50%",
            background: "var(--acc)",
            color: "#fff",
            border: "none",
            cursor: "pointer",
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
            boxShadow: "0 2px 8px rgba(0, 0, 0, 0.2)",
            transition: "transform 0.2s ease, opacity 0.2s ease",
            opacity: scrollRatio > 0.1 ? 1 : 0,
            transform: scrollRatio > 0.1 ? "scale(1)" : "scale(0.8)",
            zIndex: 10,
          }}
        >
          <ChevronUp size={18} />
        </button>
      )}
    </div>
  );
}

/**
 * 滚动位置指示器
 */
export function ScrollIndicator({
  className = "",
  style,
}: {
  className?: string;
  style?: CSSProperties;
}) {
  const { ref, scrollRatio, hasTop, hasBottom } = useScrollState();

  return (
    <div
      ref={ref}
      className={className}
      style={{
        position: "relative",
        width: 4,
        background: "var(--bg-2)",
        borderRadius: 2,
        overflow: "hidden",
        ...style,
      }}
    >
      {/* 滚动块 */}
      <div
        style={{
          position: "absolute",
          top: `${scrollRatio * 100}%`,
          left: 0,
          right: 0,
          height: 40,
          background: hasTop || hasBottom ? "var(--acc)" : "transparent",
          borderRadius: 2,
          transition: "background 0.2s ease",
          transform: `translateY(-${scrollRatio * 100}%)`,
        }}
      />
    </div>
  );
}
