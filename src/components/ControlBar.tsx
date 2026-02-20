// 顶部控制栏组件
// 包含源/目标文件夹选择、操作按钮、选项开关

interface ControlBarProps {
  sourceDir: string;
  targetDir: string;
  includeSubdirs: boolean;
  scanning: boolean;
  moving: boolean;
  hasUndo: boolean;
  hasSelection: boolean;
  previewOpen: boolean;
  onSourceDirChange: (dir: string) => void;
  onTargetDirChange: (dir: string) => void;
  onIncludeSubdirsChange: (val: boolean) => void;
  onPickSourceDir: () => void;
  onPickTargetDir: () => void;
  onScan: () => void;
  onCancelScan: () => void;
  onToggleSelectAll: () => void;
  onTogglePreview: () => void;
  onMove: () => void;
  onUndo: () => void;
}

export function ControlBar({
  sourceDir,
  targetDir,
  includeSubdirs,
  scanning,
  moving,
  hasUndo,
  hasSelection,
  previewOpen,
  onSourceDirChange,
  onTargetDirChange,
  onIncludeSubdirsChange,
  onPickSourceDir,
  onPickTargetDir,
  onScan,
  onCancelScan,
  onToggleSelectAll,
  onTogglePreview,
  onMove,
  onUndo,
}: ControlBarProps) {
  const busy = moving;

  return (
    <div
      className="shrink-0"
      style={{
        background: "var(--bg-layer)",
        borderBottom: "1px solid var(--stroke-divider)",
      }}
    >
      {/* 第一区域：两栏文件夹选择 */}
      <div
        className="flex items-stretch"
        style={{ borderBottom: "1px solid var(--stroke-divider)" }}
      >
        {/* 源文件夹 */}
        <div className="flex-1 flex items-center gap-3 px-6 py-4">
          <span
            className="text-xs font-medium whitespace-nowrap shrink-0"
            style={{ color: "var(--text-secondary)" }}
          >
            源文件夹
          </span>
          <input
            type="text"
            value={sourceDir}
            onChange={(e) => onSourceDirChange(e.target.value)}
            placeholder="选择包含图片的文件夹..."
            className="fluent-input flex-1 min-w-0"
            style={{ height: 32 }}
          />
          <button
            onClick={onPickSourceDir}
            disabled={busy}
            className="fluent-btn px-4"
            style={{ height: 32 }}
          >
            浏览...
          </button>
        </div>

        {/* 分割线 */}
        <div style={{ width: 1, background: "var(--stroke-divider)", margin: "12px 0" }} />

        {/* 目标文件夹 */}
        <div className="flex-1 flex items-center gap-3 px-6 py-4">
          <span
            className="text-xs font-medium whitespace-nowrap shrink-0"
            style={{ color: "var(--text-secondary)" }}
          >
            目标文件夹
          </span>
          <input
            type="text"
            value={targetDir}
            onChange={(e) => onTargetDirChange(e.target.value)}
            placeholder="选择输出文件夹..."
            className="fluent-input flex-1 min-w-0"
            style={{ height: 32 }}
          />
          <button
            onClick={onPickTargetDir}
            disabled={busy}
            className="fluent-btn px-4"
            style={{ height: 32 }}
          >
            浏览...
          </button>
        </div>
      </div>

      {/* 第二区域：操作按钮工具栏 */}
      <div className="flex items-center gap-2 px-6 py-3">
        {/* 扫描 / 取消扫描 */}
        {scanning ? (
          <button
            onClick={onCancelScan}
            className="fluent-btn fluent-btn-danger"
          >
            <svg className="w-3.5 h-3.5 shrink-0" fill="currentColor" viewBox="0 0 20 20">
              <path fillRule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zM8 7a1 1 0 00-1 1v4a1 1 0 001 1h4a1 1 0 001-1V8a1 1 0 00-1-1H8z" clipRule="evenodd" />
            </svg>
            取消扫描
          </button>
        ) : (
          <button
            onClick={onScan}
            disabled={busy || !sourceDir}
            className="fluent-btn fluent-btn-accent"
            style={{ minWidth: 100 }}
          >
            <svg className="w-3.5 h-3.5 shrink-0" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
              <path strokeLinecap="round" strokeLinejoin="round" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
            </svg>
            扫描图片
          </button>
        )}

        {/* 预览移动 */}
        <button
          onClick={onTogglePreview}
          disabled={busy || scanning || !hasSelection}
          className="fluent-btn"
          title={previewOpen ? "收起预览面板" : "打开预览面板"}
        >
          <svg className="w-3.5 h-3.5 shrink-0" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
            <path strokeLinecap="round" strokeLinejoin="round" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
            <path strokeLinecap="round" strokeLinejoin="round" d="M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z" />
          </svg>
          预览移动
        </button>

        {/* 执行移动 */}
        <button
          onClick={onMove}
          disabled={busy || scanning || !hasSelection || !targetDir}
          className="fluent-btn fluent-btn-success"
        >
          {moving ? (
            <>
              <LoadingSpinner />
              移动中...
            </>
          ) : (
            <>
              <svg className="w-3.5 h-3.5 shrink-0" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
                <path strokeLinecap="round" strokeLinejoin="round" d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z" />
              </svg>
              执行移动
            </>
          )}
        </button>

        {/* 撤销移动 */}
        <button
          onClick={onUndo}
          disabled={busy || scanning || !hasUndo}
          className="fluent-btn fluent-btn-warning"
        >
          <svg className="w-3.5 h-3.5 shrink-0" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
            <path strokeLinecap="round" strokeLinejoin="round" d="M3 10h10a8 8 0 018 8v2M3 10l6 6m-6-6l6-6" />
          </svg>
          撤销移动
        </button>

        {/* 分割线 */}
        <div style={{ width: 1, height: 24, background: "var(--stroke-divider)", margin: "0 4px" }} />

        {/* 全选/取消全选 */}
        <button
          onClick={onToggleSelectAll}
          disabled={busy || scanning}
          className="fluent-btn"
        >
          <svg className="w-3.5 h-3.5 shrink-0" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
            <path strokeLinecap="round" strokeLinejoin="round" d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
          </svg>
          {hasSelection ? "取消全选" : "全选/取消"}
        </button>

        {/* 包含子文件夹选项 */}
        <label
          className="flex items-center gap-2 cursor-pointer select-none ml-2"
          style={{ color: "var(--text-secondary)" }}
        >
          <input
            type="checkbox"
            checked={includeSubdirs}
            onChange={(e) => onIncludeSubdirsChange(e.target.checked)}
            className="fluent-checkbox"
          />
          <span className="text-xs">包含子文件夹</span>
        </label>

        {/* 扫描进度提示 */}
        {scanning && (
          <span
            className="flex items-center gap-1.5 text-xs ml-1"
            style={{ color: "var(--text-secondary)" }}
          >
            <LoadingSpinner />
            扫描中...
          </span>
        )}
      </div>
    </div>
  );
}

function LoadingSpinner() {
  return (
    <svg className="animate-spin h-3.5 w-3.5 shrink-0" viewBox="0 0 24 24" fill="none">
      <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4" />
      <path
        className="opacity-75"
        fill="currentColor"
        d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
      />
    </svg>
  );
}
