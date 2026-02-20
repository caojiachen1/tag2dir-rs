// 图片卡片组件
// 展示单张图片的缩略图、文件名、标签和人物选择

import { memo } from "react";
import type { CSSProperties } from "react";
import type { ImageInfo, ImageStatus } from "../types";

interface ImageCardProps {
  image: ImageInfo;
  selected: boolean;
  onToggleSelect: (id: string) => void;
  onPersonChange: (id: string, person: string) => void;
}

export const ImageCard = memo(function ImageCard({
  image,
  selected,
  onToggleSelect,
  onPersonChange,
}: ImageCardProps) {
  const statusInfo = getStatusInfo(image.status);
  const isMoved = image.status === "Moved";

  return (
    <div
      className={`fluent-card relative overflow-hidden flex flex-col cursor-pointer h-full ${selected ? "selected" : ""}`}
      style={{
        opacity: isMoved ? 0.45 : 1,
        transition: "opacity 0.2s, background 0.15s, border-color 0.15s, box-shadow 0.15s",
      }}
      onClick={() => onToggleSelect(image.id)}
    >
      {/* 选中指示器（左上角圆形） */}
      <div className="absolute top-2 left-2 z-10">
        <div
          style={{
            width: 18,
            height: 18,
            borderRadius: "50%",
            border: selected
              ? "2px solid rgba(0,120,212,0.9)"
              : "1.5px solid rgba(255,255,255,0.4)",
            background: selected
              ? "var(--accent)"
              : "rgba(0,0,0,0.5)",
            backdropFilter: "blur(4px)",
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
            transition: "all 0.15s",
            boxShadow: selected ? "0 0 6px rgba(0,120,212,0.6)" : "none",
          }}
        >
          {selected && (
            <svg width="10" height="10" viewBox="0 0 20 20" fill="white">
              <path fillRule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clipRule="evenodd" />
            </svg>
          )}
        </div>
      </div>

      {/* 状态徽标（右上角） */}
      {image.status !== "Scanned" && image.status !== "Ready" && (
        <div className="absolute top-2 right-2 z-10">
          <span
            style={{
              fontSize: 10,
              padding: "2px 6px",
              borderRadius: 999,
              fontWeight: 600,
              letterSpacing: "0.02em",
              ...statusInfo.style,
            }}
          >
            {statusInfo.label}
          </span>
        </div>
      )}

      {/* 缩略图区域 */}
      <div
        className="aspect-square flex items-center justify-center overflow-hidden"
        style={{ background: "rgba(0,0,0,0.35)" }}
      >
        {image.thumbnail ? (
          <img
            src={image.thumbnail}
            alt={image.filename}
            className="max-w-full max-h-full object-contain"
            loading="lazy"
            draggable={false}
          />
        ) : (
          <div className="flex flex-col items-center gap-1.5" style={{ color: "var(--text-disabled)" }}>
            <svg className="w-9 h-9" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={0.8}>
              <path strokeLinecap="round" strokeLinejoin="round" d="M4 16l4.586-4.586a2 2 0 012.828 0L16 16m-2-2l1.586-1.586a2 2 0 012.828 0L20 14m-6-6h.01M6 20h12a2 2 0 002-2V6a2 2 0 00-2-2H6a2 2 0 00-2 2v12a2 2 0 002 2z" />
            </svg>
            <span style={{ fontSize: 9 }}>无预览</span>
          </div>
        )}
      </div>

      {/* 内容分隔线 */}
      <div style={{ height: 1, background: "var(--stroke-divider)" }} />

      {/* 卡片信息区 */}
      <div className="flex-1 flex flex-col" style={{ padding: "10px 12px 12px" }}>
        {/* 文件名 */}
        <p
          className="truncate font-medium leading-tight"
          title={image.filename}
          style={{ fontSize: 11, color: "var(--text-primary)", marginBottom: 8 }}
        >
          {image.filename}
        </p>

        {/* 关键字标签 (固定高度，防止卡片跳变) */}
        <div className="flex flex-wrap items-center gap-1.5" style={{ marginBottom: 10, minHeight: 18 }}>
          {image.keywords.length > 0 ? (
            <>
              {image.keywords.slice(0, 3).map((kw, i) => (
                <span
                  key={i}
                  style={{
                    fontSize: 9,
                    padding: "2px 6px",
                    borderRadius: 3,
                    background: "rgba(255,255,255,0.06)",
                    border: "1px solid rgba(255,255,255,0.08)",
                    color: "var(--text-secondary)",
                    lineHeight: "11px",
                  }}
                >
                  {kw}
                </span>
              ))}
              {image.keywords.length > 3 && (
                <span style={{ fontSize: 9, color: "var(--text-tertiary)" }}>
                  +{image.keywords.length - 3}
                </span>
              )}
            </>
          ) : (
            <span style={{ fontSize: 9, color: "var(--text-disabled)", paddingLeft: 2 }}>
              无标签
            </span>
          )}
        </div>

        {/* 人物下拉框 */}
        <div className="mt-auto">
          <select
            value={image.selected_person ?? ""}
            onChange={(e) => {
              e.stopPropagation();
              if (e.target.value) onPersonChange(image.id, e.target.value);
            }}
            onClick={(e) => e.stopPropagation()}
            disabled={image.persons.length === 0}
            className="fluent-select w-full"
            style={{ height: 28 }}
          >
            {image.persons.length === 0 ? (
              <option value="">无人物标签</option>
            ) : (
              <>
                <option value="" disabled>选择人物...</option>
                {image.persons.map((person) => (
                  <option key={person} value={person}>{person}</option>
                ))}
              </>
            )}
          </select>
        </div>
      </div>
    </div>
  );
});

function getStatusInfo(status: ImageStatus): {
  label: string;
  style: CSSProperties;
} {
  if (status === "Scanned") return { label: "已扫描", style: { background: "rgba(255,255,255,0.1)", color: "rgba(255,255,255,0.6)" } };
  if (status === "Ready") return { label: "就绪", style: { background: "var(--accent-bg)", color: "var(--accent-text)" } };
  if (status === "Moving") return { label: "移动中", style: { background: "var(--warning-bg)", color: "var(--warning)" } };
  if (status === "Moved") return { label: "✓ 已移动", style: { background: "var(--success-bg)", color: "var(--success)" } };
  if (typeof status === "object" && "Error" in status)
    return { label: "错误", style: { background: "var(--danger-bg)", color: "var(--danger)" } };
  return { label: "", style: {} };
}
