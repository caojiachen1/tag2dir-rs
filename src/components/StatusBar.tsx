// 底部状态栏组件
// 显示统计信息和导航按钮

import type { ReactNode } from "react";

interface StatusBarProps {
  totalImages: number;
  personCount: number;
  selectedCount: number;
  statusMessage: string;
  onScrollToTop: () => void;
  onScrollToBottom: () => void;
}

export function StatusBar({
  totalImages,
  personCount,
  selectedCount,
  statusMessage,
  onScrollToTop,
  onScrollToBottom,
}: StatusBarProps) {
  return (
    <div
      className="flex items-center justify-between shrink-0"
      style={{
        background: "var(--bg-layer)",
        borderTop: "1px solid var(--stroke-divider)",
        padding: "8px 20px",
        minHeight: 44,
      }}
    >
      {/* 左侧：状态消息 */}
      <span
        className="text-xs truncate max-w-md"
        style={{ color: "var(--text-tertiary)" }}
      >
        {statusMessage}
      </span>

      {/* 右侧：导航 + 统计 */}
      <div className="flex items-center gap-2 shrink-0">
        {/* 顶部 / 底部 导航 */}
        <NavBtn onClick={onScrollToTop} title="回到顶部">
          <svg className="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={2.5}>
            <path strokeLinecap="round" strokeLinejoin="round" d="M5 15l7-7 7 7" />
          </svg>
          顶部
        </NavBtn>
        <NavBtn onClick={onScrollToBottom} title="回到底部">
          <svg className="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={2.5}>
            <path strokeLinecap="round" strokeLinejoin="round" d="M19 9l-7 7-7-7" />
          </svg>
          底部
        </NavBtn>

        {/* 竖线分隔 */}
        <div style={{ width: 1, height: 20, background: "var(--stroke-divider)", margin: "0 10px" }} />

        {/* 统计数字 */}
        <StatItem label="图片" value={totalImages} />
        <div style={{ width: 1, height: 20, background: "var(--stroke-divider)", margin: "0 8px" }} />
        <StatItem label="人物" value={personCount} />
        {selectedCount > 0 && (
          <>
            <div style={{ width: 1, height: 20, background: "var(--stroke-divider)", margin: "0 8px" }} />
            <StatItem label="已选" value={selectedCount} accent />
          </>
        )}
      </div>
    </div>
  );
}

function NavBtn({
  onClick,
  title,
  children,
}: {
  onClick: () => void;
  title: string;
  children: ReactNode;
}) {
  return (
    <button
      onClick={onClick}
      title={title}
      className="flex items-center gap-1"
      style={{
        padding: "3px 8px",
        borderRadius: 4,
        background: "var(--bg-control)",
        border: "1px solid var(--stroke-control-strong)",
        color: "var(--text-secondary)",
        fontSize: 11,
        cursor: "pointer",
        transition: "background 0.1s",
      }}
      onMouseEnter={(e) => (e.currentTarget.style.background = "var(--bg-control-hover)")}
      onMouseLeave={(e) => (e.currentTarget.style.background = "var(--bg-control)")}
    >
      {children}
    </button>
  );
}

function StatItem({
  label,
  value,
  accent,
}: {
  label: string;
  value: number;
  accent?: boolean;
}) {
  return (
    <span className="flex items-center gap-1" style={{ fontSize: 12 }}>
      <span style={{ color: "var(--text-tertiary)" }}>{label}:</span>
      <span
        style={{
          fontWeight: 600,
          fontVariantNumeric: "tabular-nums",
          color: accent ? "var(--accent-text)" : "var(--text-secondary)",
        }}
      >
        {value}
      </span>
    </span>
  );
}
