// 图片网格组件（虚拟化加载）
// 使用 react-virtuoso 实现大规模图片的高性能网格展示

import {
  forwardRef,
  useImperativeHandle,
  useRef,
  memo,
  useCallback,
} from "react";
import { VirtuosoGrid } from "react-virtuoso";
import type { ImageInfo } from "../types";
import { ImageCard } from "./ImageCard";

interface ImageGridProps {
  images: ImageInfo[];
  selectedIds: Set<string>;
  scanning: boolean;
  onToggleSelect: (id: string) => void;
  onPersonChange: (id: string, person: string) => void;
}

export interface ImageGridHandle {
  scrollToTop: () => void;
  scrollToBottom: () => void;
}

export const ImageGrid = memo(
  forwardRef<ImageGridHandle, ImageGridProps>(
    ({ images, selectedIds, scanning, onToggleSelect, onPersonChange }, ref) => {
      const virtuosoRef = useRef<any>(null);

      useImperativeHandle(ref, () => ({
        scrollToTop: () => {
          virtuosoRef.current?.scrollToIndex({ index: 0, behavior: "smooth" });
        },
        scrollToBottom: () => {
          virtuosoRef.current?.scrollToIndex({
            index: images.length - 1,
            behavior: "smooth",
          });
        },
      }));

      const renderItem = useCallback(
        (index: number) => {
          const image = images[index];
          if (!image) return null;
          return (
            <ImageCard
              key={image.id}
              image={image}
              selected={selectedIds.has(image.id)}
              onToggleSelect={onToggleSelect}
              onPersonChange={onPersonChange}
            />
          );
        },
        [images, selectedIds, onToggleSelect, onPersonChange]
      );

      if (images.length === 0 && !scanning) {
        return (
          <div className="flex items-center justify-center h-full">
            <div className="text-center" style={{ userSelect: "none" }}>
              <svg
                className="mx-auto mb-4"
                style={{ width: 64, height: 64, opacity: 0.18 }}
                fill="none"
                viewBox="0 0 24 24"
                stroke="currentColor"
                strokeWidth={0.7}
              >
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  d="M4 16l4.586-4.586a2 2 0 012.828 0L16 16m-2-2l1.586-1.586a2 2 0 012.828 0L20 14m-6-6h.01M6 20h12a2 2 0 002-2V6a2 2 0 00-2-2H6a2 2 0 00-2 2v12a2 2 0 002 2z"
                />
              </svg>
              <p className="mb-1" style={{ fontSize: 14, color: "var(--text-tertiary)", fontWeight: 500 }}>
                选择源文件夹并点击"扫描图片"开始
              </p>
              <p style={{ fontSize: 12, color: "var(--text-disabled)" }}>
                支持 JPG、PNG、WebP、TIFF 等格式
              </p>
            </div>
          </div>
        );
      }

      if (images.length === 0 && scanning) {
        return (
          <div className="flex items-center justify-center h-full">
            <div className="text-center">
              <svg
                className="mx-auto mb-3 animate-spin"
                style={{ width: 40, height: 40, color: "var(--accent)" }}
                fill="none"
                viewBox="0 0 24 24"
              >
                <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="3" />
                <path
                  className="opacity-75"
                  fill="currentColor"
                  d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
                />
              </svg>
              <p style={{ fontSize: 12, color: "var(--text-tertiary)" }}>正在扫描图片...</p>
            </div>
          </div>
        );
      }

      return (
        <VirtuosoGrid
          ref={virtuosoRef}
          totalCount={images.length}
          overscan={200}
          listClassName="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6 gap-6 p-8"
          itemContent={renderItem}
          style={{ height: "100%" }}
        />
      );
    }
  )
);

ImageGrid.displayName = "ImageGrid";
