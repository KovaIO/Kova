import { invoke } from "@tauri-apps/api/core";
import type { WorkspaceProfile } from "$types/preferences";

export async function getWorkspaceProfiles(): Promise<WorkspaceProfile[]> {
  return invoke<WorkspaceProfile[]>("get_workspace_profiles");
}

export async function saveWorkspaceProfile(
  profile: WorkspaceProfile,
): Promise<void> {
  return invoke("save_workspace_profile", { profile });
}

export async function deleteWorkspaceProfile(profileId: string): Promise<void> {
  return invoke("delete_workspace_profile", { profileId });
}

export async function applyWorkspace(profileId: string): Promise<void> {
  return invoke("apply_workspace", { profileId });
}
