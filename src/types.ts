// 类型定义 - 与 Rust 后端数据结构对应

export interface ImageInfo {
  id: string;
  path: string;
  filename: string;
  persons: string[];
  keywords: string[];
  thumbnail: string;
  selected_person: string | null;
  status: ImageStatus;
}

export type ImageStatus =
  | "Scanned"
  | "Ready"
  | "Moving"
  | "Moved"
  | { Error: string };

export interface ScanProgressEvent {
  scanned: number;
  image: ImageInfo | null;
  done: boolean;
  cancelled: boolean;
  error: string | null;
}

export interface MoveProgressEvent {
  moved_count: number;
  total: number;
  current_file: string;
  done: boolean;
  error: string | null;
}

export interface ScanStats {
  total_images: number;
  person_count: number;
  person_names: string[];
}

export interface MoveImageRequest {
  path: string;
  filename: string;
  person: string;
}

export interface MoveResult {
  moved_count: number;
  has_undo: boolean;
}

export interface UndoResult {
  restored_count: number;
  success: boolean;
}
