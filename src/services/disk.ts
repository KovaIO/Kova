import { invoke } from "@tauri-apps/api/core";
import type {
  DiskItemDetail,
  DiskScanResult,
  DiskVolumeInfo,
  ScanPreview,
} from "$types/disk";

export async function fetchDiskVolumeInfo(): Promise<DiskVolumeInfo> {
  return invoke<DiskVolumeInfo>("get_disk_volume_info");
}

export async function fetchDiskScanPreview(): Promise<ScanPreview> {
  return invoke<ScanPreview>("get_disk_scan_preview");
}

export async function fetchDiskScanResult(): Promise<DiskScanResult | null> {
  return invoke<DiskScanResult | null>("get_disk_scan_result");
}

export async function startDiskScan(): Promise<DiskScanResult> {
  return invoke<DiskScanResult>("start_disk_scan");
}

export async function fetchDiskItemDetail(
  itemId: string,
): Promise<DiskItemDetail> {
  return invoke<DiskItemDetail>("get_disk_item_detail", { itemId });
}

export async function deleteDiskItems(ids: string[]): Promise<number> {
  return invoke<number>("delete_disk_items", { ids });
}

export async function deleteAllDiskItems(): Promise<number> {
  return invoke<number>("delete_all_disk_items");
}
