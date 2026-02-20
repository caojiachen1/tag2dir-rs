import { useState, useCallback, useRef, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { open } from "@tauri-apps/plugin-dialog";
import type {
  ImageInfo,
  ScanProgressEvent,
  MoveProgressEvent,
  ScanStats,
  MoveImageRequest,
  MoveResult,
  UndoResult,
} from "./types";
import { ControlBar } from "./components/ControlBar";
import { ImageGrid } from "./components/ImageGrid";
import { StatusBar } from "./components/StatusBar";

function App() {
  // 文件夹路径
  const [sourceDir, setSourceDir] = useState("");
  const [targetDir, setTargetDir] = useState("");
  const [includeSubdirs, setIncludeSubdirs] = useState(true);

  // 图片数据
  const [images, setImages] = useState<ImageInfo[]>([]);
  const [selectedIds, setSelectedIds] = useState<Set<string>>(new Set());

  // 状态
  const [scanning, setScanning] = useState(false);
  const [moving, setMoving] = useState(false);
  const [moveProgress, setMoveProgress] = useState<MoveProgressEvent | null>(
    null
  );
  const [hasUndo, setHasUndo] = useState(false);
  const [statusMessage, setStatusMessage] = useState("就绪");

  // 虚拟列表引用
  const gridRef = useRef<{
    scrollToTop: () => void;
    scrollToBottom: () => void;
  } | null>(null);

  // 监听扫描进度事件
  useEffect(() => {
    const unlisten = listen<ScanProgressEvent>("scan-progress", (event) => {
      const data = event.payload;
      if (data.image) {
        setImages((prev) => [...prev, data.image!]);
      }
      if (data.done) {
        setScanning(false);
        if (data.cancelled) {
          setStatusMessage(`扫描已取消，已加载 ${data.scanned} 张图片`);
        } else {
          setStatusMessage(`扫描完成，共 ${data.scanned} 张图片`);
        }
      }
    });
    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  // 监听移动进度事件
  useEffect(() => {
    const unlisten = listen<MoveProgressEvent>("move-progress", (event) => {
      setMoveProgress(event.payload);
      if (event.payload.done) {
        setMoving(false);
      }
    });
    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  // 选择源文件夹
  const pickSourceDir = useCallback(async () => {
    const selected = await open({
      directory: true,
      multiple: false,
      title: "选择源文件夹",
    });
    if (selected) {
      setSourceDir(selected as string);
    }
  }, []);

  // 选择目标文件夹
  const pickTargetDir = useCallback(async () => {
    const selected = await open({
      directory: true,
      multiple: false,
      title: "选择目标文件夹",
    });
    if (selected) {
      setTargetDir(selected as string);
    }
  }, []);

  // 开始扫描
  const startScan = useCallback(async () => {
    if (!sourceDir) {
      setStatusMessage("请先选择源文件夹");
      return;
    }
    setScanning(true);
    setImages([]);
    setSelectedIds(new Set());
    setHasUndo(false);
    setStatusMessage("正在扫描...");

    try {
      await invoke<ScanStats>("scan_images", {
        sourceDir,
        includeSubdirs,
      });
    } catch (e) {
      setStatusMessage(`扫描失败: ${e}`);
      setScanning(false);
    }
  }, [sourceDir, includeSubdirs]);

  // 取消扫描
  const cancelScan = useCallback(async () => {
    try {
      await invoke("cancel_scan");
      setStatusMessage("正在取消扫描...");
    } catch (e) {
      setStatusMessage(`取消失败: ${e}`);
    }
  }, []);

  // 全选/取消全选
  const toggleSelectAll = useCallback(() => {
    setSelectedIds((prev) => {
      if (prev.size > 0) {
        return new Set();
      }
      const allIds = new Set(
        images.filter((img) => img.selected_person).map((img) => img.id)
      );
      return allIds;
    });
  }, [images]);

  // 切换单个图片选中状态
  const toggleSelect = useCallback((id: string) => {
    setSelectedIds((prev) => {
      const next = new Set(prev);
      if (next.has(id)) {
        next.delete(id);
      } else {
        next.add(id);
      }
      return next;
    });
  }, []);

  // 更新图片的选定人物
  const updatePersonSelection = useCallback((id: string, person: string) => {
    setImages((prev) =>
      prev.map((img) =>
        img.id === id
          ? { ...img, selected_person: person, status: "Ready" as const }
          : img
      )
    );
  }, []);

  // 执行移动
  const executeMove = useCallback(async () => {
    if (!targetDir) {
      setStatusMessage("请先选择目标文件夹");
      return;
    }

    const toMove = images.filter(
      (img) => selectedIds.has(img.id) && img.selected_person
    );

    if (toMove.length === 0) {
      setStatusMessage("没有可移动的图片（请确保已选择图片并设置人物）");
      return;
    }

    setMoving(true);
    setMoveProgress(null);
    setStatusMessage("正在移动文件...");

    try {
      const requests: MoveImageRequest[] = toMove.map((img) => ({
        path: img.path,
        filename: img.filename,
        person: img.selected_person!,
      }));

      const result = await invoke<MoveResult>("move_images", {
        images: requests,
        targetDir,
      });

      setHasUndo(result.has_undo);
      setStatusMessage(`移动完成，成功移动 ${result.moved_count} 个文件`);

      // 更新被移动图片的状态
      const movedPaths = new Set(toMove.map((img) => img.path));
      setImages((prev) =>
        prev.map((img) =>
          movedPaths.has(img.path)
            ? { ...img, status: "Moved" as const }
            : img
        )
      );
      setSelectedIds(new Set());
    } catch (e) {
      setStatusMessage(`移动失败: ${e}`);
    } finally {
      setMoving(false);
    }
  }, [images, selectedIds, targetDir]);

  // 撤销移动
  const undoLastMove = useCallback(async () => {
    setStatusMessage("正在撤销...");
    try {
      const result = await invoke<UndoResult>("undo_move");
      if (result.success) {
        setStatusMessage(`撤销成功，恢复了 ${result.restored_count} 个文件`);
        setHasUndo(false);
        setImages((prev) =>
          prev.map((img) =>
            img.status === "Moved"
              ? { ...img, status: "Scanned" as const }
              : img
          )
        );
      }
    } catch (e) {
      setStatusMessage(`撤销失败: ${e}`);
    }
  }, []);

  // 桶算法计算人物数
  const personBuckets = new Set<string>();
  images.forEach((img) => {
    img.persons.forEach((p) => personBuckets.add(p));
  });

  return (
    <div className="flex flex-col h-screen" style={{ background: "var(--bg-base)" }}>
      {/* 顶部控制栏 */}
      <ControlBar
        sourceDir={sourceDir}
        targetDir={targetDir}
        includeSubdirs={includeSubdirs}
        scanning={scanning}
        moving={moving}
        hasUndo={hasUndo}
        hasSelection={selectedIds.size > 0}
        onSourceDirChange={setSourceDir}
        onTargetDirChange={setTargetDir}
        onIncludeSubdirsChange={setIncludeSubdirs}
        onPickSourceDir={pickSourceDir}
        onPickTargetDir={pickTargetDir}
        onScan={startScan}
        onCancelScan={cancelScan}
        onToggleSelectAll={toggleSelectAll}
        onMove={executeMove}
        onUndo={undoLastMove}
      />

      <div className="flex-1 overflow-hidden">
        <ImageGrid
          ref={gridRef}
          images={images}
          selectedIds={selectedIds}
          scanning={scanning}
          onToggleSelect={toggleSelect}
          onPersonChange={updatePersonSelection}
        />
      </div>

      {/* 移动进度条 */}
      {moving && moveProgress && (
        <div
          style={{
            background: "var(--bg-layer)",
            borderTop: "1px solid var(--stroke-divider)",
            padding: "10px 20px",
          }}
        >
          <div className="flex items-center gap-4">
            <span
              className="text-xs font-medium whitespace-nowrap"
              style={{ color: "var(--text-secondary)" }}
            >
              正在移动素材 {moveProgress.moved_count} / {moveProgress.total}
            </span>
            <div
              className="flex-1 overflow-hidden"
              style={{
                height: 4,
                background: "rgba(255,255,255,0.1)",
                borderRadius: 999,
              }}
            >
              <div
                className="h-full progress-bar-animated transition-all duration-300"
                style={{
                  width: `${
                    moveProgress.total > 0
                      ? (moveProgress.moved_count / moveProgress.total) * 100
                      : 0
                  }%`,
                  background: "var(--accent)",
                  borderRadius: 999,
                }}
              />
            </div>
          </div>
        </div>
      )}

      <StatusBar
        totalImages={images.length}
        personCount={personBuckets.size}
        selectedCount={selectedIds.size}
        statusMessage={statusMessage}
        onScrollToTop={() => gridRef.current?.scrollToTop()}
        onScrollToBottom={() => gridRef.current?.scrollToBottom()}
      />
    </div>
  );
}

export default App;
