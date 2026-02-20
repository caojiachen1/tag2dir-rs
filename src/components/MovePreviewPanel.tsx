interface MovePreviewGroup {
  person: string;
  count: number;
  items: {
    id: string;
    filename: string;
    thumbnail: string;
  }[];
}

interface MovePreviewPanelProps {
  targetDir: string;
  selectedCount: number;
  validCount: number;
  unassignedCount: number;
  groups: MovePreviewGroup[];
  onClose: () => void;
}

export function MovePreviewPanel({
  targetDir,
  selectedCount,
  validCount,
  unassignedCount,
  groups,
  onClose,
}: MovePreviewPanelProps) {
  return (
    <div
      className="shrink-0"
      style={{
        background: "var(--bg-layer)",
        borderTop: "1px solid var(--stroke-divider)",
      }}
    >
      <div
        className="flex items-center justify-between gap-3 px-5 py-2.5"
        style={{
          background: "var(--bg-layer-alt)",
          borderBottom: "1px solid var(--stroke-divider)",
        }}
      >
        <span className="text-xs" style={{ color: "var(--text-secondary)" }}>
          移动预览：{validCount} / {selectedCount} 张可移动
          {unassignedCount > 0 ? `（${unassignedCount} 张未设置人物）` : ""}
        </span>
        <button
          type="button"
          className="p-1.5 hover:bg-white/10 rounded-md transition-colors"
          style={{ color: "var(--text-secondary)" }}
          onClick={onClose}
          title="关闭预览"
        >
          <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
            <path strokeLinecap="round" strokeLinejoin="round" d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>
      </div>

      <div className="px-5 py-3" style={{ maxHeight: 360, overflowY: "auto" }}>
        {!targetDir ? (
          <p className="text-xs" style={{ color: "var(--warning)" }}>
            请选择目标文件夹后再确认移动路径。
          </p>
        ) : groups.length === 0 ? (
          <p className="text-xs" style={{ color: "var(--text-tertiary)" }}>
            当前没有可预览的移动项，请先选择图片并设置人物。
          </p>
        ) : (
          <div className="flex flex-col gap-4">
            {groups.map((group) => {
              const folderPath = `${targetDir}\\${group.person}`;
              return (
                <div
                  key={group.person}
                  style={{
                    background: "var(--bg-subtle)",
                    border: "1px solid var(--stroke-divider)",
                    borderRadius: 6,
                    padding: "10px 12px",
                  }}
                >
                  <div className="flex items-center justify-between gap-3 mb-3">
                    <code
                      className="text-xs truncate"
                      style={{ color: "var(--accent-text)" }}
                      title={folderPath}
                    >
                      {folderPath}
                    </code>
                    <span className="text-xs shrink-0" style={{ color: "var(--text-secondary)" }}>
                      {group.count} 张
                    </span>
                  </div>

                  <div className="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 gap-3">
                    {group.items.slice(0, 15).map((item) => (
                      <div
                        key={`${group.person}-${item.id}`}
                        style={{
                          background: "var(--bg-control)",
                          border: "1px solid var(--stroke-control)",
                          borderRadius: 6,
                          overflow: "hidden",
                        }}
                        title={item.filename}
                      >
                        <div
                          className="w-full flex items-center justify-center"
                          style={{ height: 108, background: "rgba(0,0,0,0.35)" }}
                        >
                          {item.thumbnail ? (
                            <img
                              src={item.thumbnail}
                              alt={item.filename}
                              className="w-full h-full object-contain"
                              draggable={false}
                            />
                          ) : (
                            <span className="text-[11px]" style={{ color: "var(--text-disabled)" }}>
                              无预览
                            </span>
                          )}
                        </div>
                        <div
                          className="truncate"
                          style={{
                            fontSize: 11,
                            color: "var(--text-secondary)",
                            padding: "5px 7px",
                          }}
                        >
                          {item.filename}
                        </div>
                      </div>
                    ))}
                  </div>

                  {group.items.length > 15 && (
                    <div className="text-[11px] mt-2" style={{ color: "var(--text-tertiary)" }}>
                      仅展示前 15 张，另外还有 {group.items.length - 15} 张...
                    </div>
                  )}
                </div>
              );
            })}
          </div>
        )}
      </div>
    </div>
  );
}
