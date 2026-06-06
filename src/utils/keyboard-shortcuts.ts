import { platform } from "@tauri-apps/plugin-os";

import type { Shortcut } from "$types/preferences";

export type ShortcutCatalogEntry = {
  action: string;
  label: string;
  section: string;
};

export const SHORTCUT_CATALOG: ShortcutCatalogEntry[] = [
  {
    action: "apply_workspace",
    label: "Apply Workspace",
    section: "Apply Workspace",
  },
  {
    action: "open_window_switcher",
    label: "Open window switcher",
    section: "Window Switcher",
  },
  {
    action: "previous_window",
    label: "Select previous window",
    section: "Window Switcher",
  },
  {
    action: "search_window_switcher",
    label: "Search",
    section: "Window Switcher",
  },
  { action: "expand_tabs", label: "Expand tabs", section: "Window Switcher" },
  {
    action: "collapse_tabs",
    label: "Collapse tabs",
    section: "Window Switcher",
  },
  {
    action: "open_clipboard_history",
    label: "Open clipboard history",
    section: "Clipboard History",
  },
  {
    action: "search_clipboard_history",
    label: "Search clipboard history",
    section: "Clipboard History",
  },
  {
    action: "open_monitor",
    label: "Open system monitoring dashboard",
    section: "Windows",
  },
  {
    action: "open_menubar_popover",
    label: "Open menubar popover",
    section: "Windows",
  },
];

export const MODIFIER_KEYS = new Set(["Control", "Shift", "Alt", "Meta"]);

const WINDOWS_BLOCKED = new Set([
  "alt+tab",
  "ctrl+tab",
  "ctrl+space",
  "ctrl+alt+m",
]);

const KEY_FROM_CODE: Record<string, string> = {
  Backquote: "`",
  Minus: "-",
  Equal: "=",
  BracketLeft: "[",
  BracketRight: "]",
  Backslash: "\\",
  Semicolon: ";",
  Quote: "'",
  Comma: ",",
  Period: ".",
  Slash: "/",
};

const KEY_DISPLAY: Record<string, string> = {
  " ": "Space",
  Space: "Space",
  Tab: "Tab",
  Enter: "Enter",
  Escape: "Escape",
  Backspace: "Backspace",
  Delete: "Delete",
  ArrowUp: "↑",
  ArrowDown: "↓",
  ArrowLeft: "←",
  ArrowRight: "→",
  "`": "`",
  "~": "`",
  Backquote: "`",
};

export function snakeToPascal(value: string): string {
  return value
    .split("_")
    .map((part) => part.charAt(0).toUpperCase() + part.slice(1))
    .join("");
}

export function actionToSnake(value: string): string {
  return value
    .replace(/([a-z0-9])([A-Z])/g, "$1_$2")
    .replace(/([A-Z])([A-Z][a-z])/g, "$1_$2")
    .toLowerCase();
}

export function actionsMatch(a: string, b: string): boolean {
  return actionToSnake(a) === actionToSnake(b);
}

export function partDisplayLabel(part: string): string {
  if (part === "Super") {
    return platform() === "macos" ? "⌘" : "Win";
  }
  return part;
}

export function formatShortcutParts(parts: string[]): string {
  return parts.join(" + ");
}

function modifiersFromEvent(event: KeyboardEvent): string[] {
  const parts: string[] = [];
  if (event.ctrlKey) parts.push("Ctrl");
  if (event.altKey) parts.push("Alt");
  if (event.shiftKey) parts.push("Shift");
  if (event.metaKey) parts.push("Super");
  return parts;
}

function mainKeyFromEvent(event: KeyboardEvent): string | null {
  if (event.code === "Backquote" || event.key === "`" || event.key === "~") {
    return "`";
  }

  if (KEY_FROM_CODE[event.code]) {
    return KEY_FROM_CODE[event.code]!;
  }

  return normalizeMainKey(event.key);
}

export function normalizeCombo(parts: string[]): string {
  if (parts.length === 0) return "";

  const key = parts[parts.length - 1]!.toLowerCase();
  const modifiers = parts
    .slice(0, -1)
    .map((part) => part.toLowerCase())
    .sort();

  return [...modifiers, key].join("+");
}

export function isWindowsBlockedCombo(parts: string[]): boolean {
  if (platform() !== "windows") return false;
  return WINDOWS_BLOCKED.has(normalizeCombo(parts));
}

function normalizeMainKey(key: string): string | null {
  if (KEY_DISPLAY[key]) return KEY_DISPLAY[key]!;

  if (key.length === 1) {
    if (/^[A-Za-z0-9]$/.test(key)) {
      return key.toUpperCase();
    }
    return key;
  }

  if (/^F\d{1,2}$/i.test(key)) {
    return key.toUpperCase();
  }

  return null;
}

export function partsFromKeyboardEvent(event: KeyboardEvent): string[] | null {
  if (MODIFIER_KEYS.has(event.key)) {
    return modifiersFromEvent(event);
  }

  const main = mainKeyFromEvent(event);
  if (!main) return null;

  const parts = [...modifiersFromEvent(event), main];

  if (parts.length < 2 || parts.length > 3) {
    return null;
  }

  return parts;
}

export function previewPartsFromEvent(event: KeyboardEvent): string[] {
  if (MODIFIER_KEYS.has(event.key)) {
    return modifiersFromEvent(event);
  }

  const main = mainKeyFromEvent(event);
  if (main) {
    return [...modifiersFromEvent(event), main];
  }

  return modifiersFromEvent(event);
}

export type ShortcutValidationResult =
  | { ok: true; parts: string[]; formatted: string }
  | { ok: false; message: string };

export function validateShortcutCapture(
  parts: string[],
  shortcuts: Shortcut[],
  editingAction: string,
): ShortcutValidationResult {
  if (parts.length < 2) {
    return { ok: false, message: "Use at least two keys (e.g. Ctrl + Space)." };
  }

  if (parts.length > 3) {
    return { ok: false, message: "Use at most three keys." };
  }

  if (isWindowsBlockedCombo(parts)) {
    return {
      ok: false,
      message: "This shortcut is reserved by Windows and cannot be used.",
    };
  }

  const formatted = formatShortcutParts(parts);
  const duplicate = shortcuts.find(
    (shortcut) =>
      !actionsMatch(shortcut.action, editingAction) &&
      normalizeCombo(shortcut.keys.split("+").map((p) => p.trim())) ===
        normalizeCombo(parts),
  );

  if (duplicate) {
    return { ok: false, message: "This shortcut is already assigned." };
  }

  return { ok: true, parts, formatted };
}

export function groupCatalogBySection(
  catalog: ShortcutCatalogEntry[],
): { title: string; shortcuts: ShortcutCatalogEntry[] }[] {
  const sections = new Map<string, ShortcutCatalogEntry[]>();

  for (const entry of catalog) {
    const list = sections.get(entry.section) ?? [];
    list.push(entry);
    sections.set(entry.section, list);
  }

  return Array.from(sections.entries()).map(([title, shortcuts]) => ({
    title,
    shortcuts,
  }));
}

export const CATALOG_SECTIONS = groupCatalogBySection(SHORTCUT_CATALOG);

export function mergeShortcutsWithCatalog(
  saved: Shortcut[] | undefined,
): { action: string; label: string; keys: string | null }[] {
  return SHORTCUT_CATALOG.map((entry) => {
    const match = saved?.find((shortcut) =>
      actionsMatch(shortcut.action, entry.action),
    );

    return {
      action: entry.action,
      label: entry.label,
      keys: match?.keys ?? null,
    };
  });
}

export function shortcutsForApi(shortcuts: Shortcut[]): Shortcut[] {
  return shortcuts.map((shortcut) => ({
    action: snakeToPascal(actionToSnake(shortcut.action)),
    keys: shortcut.keys,
  }));
}

export function normalizeStoredShortcuts(shortcuts: Shortcut[]): Shortcut[] {
  return shortcuts.map((shortcut) => ({
    action: actionToSnake(shortcut.action),
    keys: shortcut.keys,
  }));
}
