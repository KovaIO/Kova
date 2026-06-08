export type DiskCategory =
  | "system"
  | "browsers"
  | "development"
  | "applications"
  | "storage"
  | "other";

export type DiskSafety = "safe" | "caution" | "unsafe";

export interface DiskVolumeInfo {
  mount_path: string;
  label: string;
  total_bytes: number;
  used_bytes: number;
  available_bytes: number;
  used_percent: number;
  last_scan_at: number | null;
}

export interface ScanPreviewCategory {
  category: DiskCategory;
  label: string;
  description: string;
  target_count: number;
}

export interface ScanPreview {
  mount_path: string;
  categories: ScanPreviewCategory[];
}

export interface DiskScanProgress {
  progress: number;
  message: string;
  phase: string;
}

export interface DiskScanItem {
  id: string;
  name: string;
  path: string;
  size_bytes: number;
  category: DiskCategory;
  is_dir: boolean;
  item_count: number;
  safety: DiskSafety;
  safety_reason: string;
}

export interface DiskCategoryGroup {
  category: DiskCategory;
  label: string;
  total_bytes: number;
  items: DiskScanItem[];
}

export interface DiskScanResult {
  scanned_at: number;
  mount_path: string;
  total_bytes: number;
  used_bytes: number;
  available_bytes: number;
  total_reclaimable: number;
  categories: DiskCategoryGroup[];
}

export interface DiskItemChild {
  name: string;
  path: string;
  size_bytes: number;
  is_dir: boolean;
}

export interface DiskItemDetail {
  item: DiskScanItem;
  children: DiskItemChild[];
}

export const CATEGORY_ICONS: Record<DiskCategory, string> = {
  system: "settings",
  browsers: "globe",
  development: "code",
  applications: "app",
  storage: "folder",
  other: "layers",
};

export function safetyLabel(safety: DiskSafety): string {
  switch (safety) {
    case "safe":
      return "Safe to delete";
    case "caution":
      return "Review before deleting";
    case "unsafe":
      return "Not recommended";
  }
}

export function formatLastScan(unix: number | null | undefined): string {
  if (!unix) return "Never";
  const date = new Date(unix * 1000);
  return date.toLocaleString([], {
    month: "short",
    day: "numeric",
    hour: "2-digit",
    minute: "2-digit",
  });
}
