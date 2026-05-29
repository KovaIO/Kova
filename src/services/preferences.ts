import { invoke } from "@tauri-apps/api/core";

export async function updateWindowManager(value: number) {
  await invoke("update_window_manager_preferences", {
    prefs: {
      snap_to_edges: true,
      remember_position: true,
      hide_on_focus_loss: true,
      opacity: value,
    },
  });
}
