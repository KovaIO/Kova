export const CLIPBOARD_HISTORY_UNLIMITED = 0;

export const CLIPBOARD_HISTORY_OPTIONS = [
  { value: 10, label: "10 items" },
  { value: 25, label: "25 items" },
  { value: 50, label: "50 items" },
  { value: 100, label: "100 items" },
  {
    value: CLIPBOARD_HISTORY_UNLIMITED,
    label: "Unlimited",
    proOnly: true,
  },
] as const;

export type ClipboardContentType = "text" | "image";

export interface ClipboardItem {
  id: number;
  content_type: ClipboardContentType;
  text_content: string | null;
  image_path: string | null;
  source_app: string | null;
  source_app_icon: string | null;
  source_app_path: string | null;
  image_width: number | null;
  image_height: number | null;
  image_size: number | null;
  image_filename: string | null;
  created_at: number;
}

export function formatHistoryLimit(value: number): string {
  if (value === CLIPBOARD_HISTORY_UNLIMITED) {
    return "Unlimited";
  }

  return `${value} items`;
}

export function formatClipboardTime(unixSeconds: number): string {
  const date = new Date(unixSeconds * 1000);
  return date.toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" });
}

export function previewText(item: ClipboardItem, maxLen = 220): string {
  if (item.content_type !== "text" || !item.text_content) {
    return "";
  }

  const normalized = item.text_content.replace(/\s+/g, " ").trim();
  if (normalized.length <= maxLen) {
    return normalized;
  }

  return `${normalized.slice(0, maxLen)}…`;
}

export function isRasterIcon(
  icon: string | null | undefined,
): icon is string {
  return !!icon && icon !== "system" && icon !== "terminal";
}

export function isUrl(item: ClipboardItem): boolean {
  const text = item.text_content?.trim() ?? "";
  return text.startsWith("http://") || text.startsWith("https://");
}

export function formatFileSize(bytes: number | null | undefined): string {
  if (!bytes) return "—";
  if (bytes >= 1024 * 1024) {
    return `${(bytes / (1024 * 1024)).toFixed(2)} MB`;
  }
  if (bytes >= 1024) {
    return `${(bytes / 1024).toFixed(2)} KB`;
  }
  return `${bytes} B`;
}

export function imageFolder(path: string | null | undefined): string {
  if (!path) return "";
  const normalized = path.replace(/\\/g, "/");
  const index = normalized.lastIndexOf("/");
  if (index <= 0) return normalized;
  return `${normalized.slice(0, index + 1)}`;
}
