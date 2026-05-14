# 滚动优化系统

DeviceDeck 的滚动条优化系统提供了精致、现代的滚动体验，与应用的整体工业美学设计保持一致。

## 特性

### 🎨 视觉设计
- **渐变滚动条** - 使用微妙的渐变效果，增加层次感
- **动态显示** - 鼠标悬停时才显示滚动条，保持界面简洁
- **主题适配** - 自动适配深色/浅色主题
- **精致轨道** - 半透明轨道，不会喧宾夺主

### ✨ 交互效果
- **滚动遮罩** - 顶部/底部渐变遮罩，提示可滚动内容
- **悬停反馈** - 滚动条悬停时颜色变化
- **按下反馈** - 点击滚动条时使用主题色
- **平滑滚动** - 支持平滑滚动动画

### ♿ 无障碍支持
- **键盘导航** - 完整的键盘导航支持
- **减少动画** - 支持 `prefers-reduced-motion`
- **高对比度** - 支持 `forced-colors` 模式
- **焦点样式** - 清晰的焦点指示器

## 使用方法

### 1. 基础滚动容器

```tsx
import { ScrollFade } from "./components/ui/ScrollFade";

function MyComponent() {
  return (
    <ScrollFade maxHeight="400px">
      {/* 你的内容 */}
      <div>Item 1</div>
      <div>Item 2</div>
      {/* ... */}
    </ScrollFade>
  );
}
```

### 2. 使用自定义 Hook

```tsx
import { useScrollState } from "../hooks/useScrollState";

function MyComponent() {
  const { ref, hasTop, hasBottom, scrollRatio, isScrolling } = useScrollState({
    threshold: 10,      // 滚动阈值
    scrollingDelay: 150, // 滚动停止延迟
  });

  return (
    <div ref={ref} style={{ overflowY: "auto", maxHeight: "400px" }}>
      {/* 渐变遮罩 */}
      {hasTop && <div className="scroll-top-mask" />}

      {/* 内容 */}
      <div>Content...</div>

      {hasBottom && <div className="scroll-bottom-mask" />}
    </div>
  );
}
```

### 3. 带滚动到顶部按钮

```tsx
import { ScrollFade } from "./components/ui/ScrollFade";

function LongList() {
  return (
    <ScrollFade maxHeight="600px" showScrollToTop>
      {/* 长列表内容 */}
    </ScrollFade>
  );
}
```

### 4. 增强的 Dropdown

```tsx
import { Dropdown } from "./components/ui/Dropdown";

function MyForm() {
  const [value, setValue] = useState("");

  return (
    <Dropdown
      value={value}
      onChange={setValue}
      options={[
        { value: "1", label: "Option 1" },
        { value: "2", label: "Option 2" },
        // ...
      ]}
    />
  );
}
```

**Dropdown 新特性：**
- 键盘导航 (↑↓箭头、Enter、Escape、Home、End)
- 选项高亮显示
- 自动滚动到选中项
- ARIA 无障碍属性

## CSS 类名

### 全局滚动条样式

所有滚动条自动应用以下样式：
- Webkit 浏览器：渐变滚动条，动态显示
- Firefox：细滚动条，悬停时颜色变化

### 可用的 CSS 类

```css
/* 优化滚动性能 */
.optimize-scroll {
  -webkit-overflow-scrolling: touch;
  will-change: scroll-position;
}

/* 滚动容器 */
.scroll-container {
  contain: layout style;
  overflow-anchor: none;
}

/* 滚动状态指示 */
.has-scroll-top::before { /* 顶部遮罩 */ }
.has-scroll-bottom::after { /* 底部遮罩 */ }

/* 脉冲动画 (用于重要通知) */
.scrollbar-pulse::-webkit-scrollbar-thumb {
  animation: scrollbar-pulse 2s ease-in-out infinite;
}
```

## 自定义配置

### 修改滚动条宽度

在 `index.css` 中：

```css
/* 全局滚动条宽度 */
::-webkit-scrollbar {
  width: 8px;  /* 默认 6px */
}

/* 内容区域滚动条 */
.content::-webkit-scrollbar {
  width: 10px; /* 默认 8px */
}

/* 日志表格滚动条 */
.log-table > div:last-child::-webkit-scrollbar {
  width: 12px; /* 默认 10px */
}
```

### 修改渐变遮罩高度

```tsx
<ScrollFade fadeHeight={48}> {/* 默认 32px */}
  {/* 内容 */}
</ScrollFade>
```

### 修改滚动阈值

```tsx
const { ref, hasTop, hasBottom } = useScrollState({
  threshold: 20, // 默认 10px
});
```

## 响应式行为

### 移动设备 (触摸)
- 滚动条宽度自动减小
- 优化触摸滚动体验

### 超宽屏幕 (>1920px)
- 滚动条宽度自动增大
- 更好的视觉比例

### 减少动画模式
- 禁用所有滚动动画
- 禁用滚动条过渡效果
- 禁用行悬停动画

## 性能优化

1. **被动事件监听** - 使用 `{ passive: true }` 优化滚动性能
2. **ResizeObserver** - 监听内容变化，自动更新滚动状态
3. **防抖处理** - 滚动停止检测使用防抖，避免频繁更新
4. **CSS containment** - 使用 `contain` 属性优化渲染性能
5. **will-change** - 提示浏览器优化滚动动画

## 浏览器兼容性

| 特性 | Chrome | Firefox | Safari | Edge |
|------|--------|---------|--------|------|
| Webkit 滚动条 | ✅ | ❌ | ✅ | ✅ |
| Firefox 滚动条 | ❌ | ✅ | ❌ | ❌ |
| 平滑滚动 | ✅ | ✅ | ✅ | ✅ |
| ResizeObserver | ✅ | ✅ | ✅ | ✅ |
| CSS Containment | ✅ | ✅ | ✅ | ✅ |

## 最佳实践

1. **使用语义化容器** - 为滚动容器设置明确的 `maxHeight`
2. **避免嵌套滚动** - 不要在滚动容器内嵌套另一个滚动容器
3. **性能考虑** - 大列表使用虚拟滚动
4. **无障碍** - 确保键盘可以访问所有可滚动内容
5. **测试** - 在不同设备和浏览器上测试滚动行为
