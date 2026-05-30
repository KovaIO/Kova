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

export function formatHistoryLimit(value: number): string {
  if (value === CLIPBOARD_HISTORY_UNLIMITED) {
    return "Unlimited";
  }

  return `${value} items`;
}
