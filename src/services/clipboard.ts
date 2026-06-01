import { invoke } from "@tauri-apps/api/core";
import type { ClipboardItem } from "$types/clipboard";

export async function fetchClipboardHistory(
  search?: string,
): Promise<ClipboardItem[]> {
  return invoke<ClipboardItem[]>("get_clipboard_history", {
    search: search?.trim() || null,
  });
}

export async function pasteClipboardItem(id: number): Promise<void> {
  await invoke("paste_clipboard_item", { id });
}

export async function copyClipboardItem(id: number): Promise<void> {
  await invoke("copy_clipboard_item", { id });
}

export async function pastePlainClipboardItem(id: number): Promise<void> {
  await invoke("paste_plain_clipboard_item", { id });
}

export async function openClipboardUrl(id: number): Promise<void> {
  await invoke("open_clipboard_url", { id });
}

export async function revealClipboardItem(id: number): Promise<void> {
  await invoke("reveal_clipboard_item", { id });
}

export async function previewClipboardItem(id: number): Promise<void> {
  await invoke("preview_clipboard_item", { id });
}

export async function deleteClipboardItem(id: number): Promise<void> {
  await invoke("delete_clipboard_item", { id });
}

export async function clearClipboardHistory(): Promise<void> {
  await invoke("clear_clipboard_history");
}
