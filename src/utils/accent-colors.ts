import type { AccentColor } from "$types/preferences";

export const ACCENT_COLORS: {
  value: AccentColor;
  label: string;
  color: string;
}[] = [
  {
    value: "purple",
    label: "Purple",
    color: "#9b7cf0",
  },
  {
    value: "blue",
    label: "Blue",
    color: "#8cb4f5",
  },
  {
    value: "green",
    label: "Green",
    color: "#9adbb3",
  },
  {
    value: "orange",
    label: "Orange",
    color: "#f5b793",
  },
  {
    value: "white",
    label: "White",
    color: "#e1e3e6",
  },
];

export function applyAccentColor(color: AccentColor) {
  const root = document.documentElement;

  switch (color) {
    case "blue":
      root.style.setProperty("--color-accent", "#8cb4f5");
      root.style.setProperty("--color-accent-hover", "#a7c7f8");
      root.style.setProperty("--color-accent-soft", "rgba(140,180,245,0.10)");
      root.style.setProperty("--color-accent-border", "rgba(140,180,245,0.25)");
      root.style.setProperty("--color-border-accent", "rgba(140,180,245,0.20)");
      root.style.setProperty("--color-accent-text", "#ebebf0");
      break;

    case "green":
      root.style.setProperty("--color-accent", "#9adbb3");
      root.style.setProperty("--color-accent-hover", "#b4e5c7");
      root.style.setProperty("--color-accent-soft", "rgba(154,219,179,0.10)");
      root.style.setProperty("--color-accent-border", "rgba(154,219,179,0.25)");
      root.style.setProperty("--color-border-accent", "rgba(154,219,179,0.20)");
      root.style.setProperty("--color-accent-text", "#1f2937");
      break;

    case "orange":
      root.style.setProperty("--color-accent", "#f5b793");
      root.style.setProperty("--color-accent-hover", "#f8c7ab");
      root.style.setProperty("--color-accent-soft", "rgba(245,183,147,0.10)");
      root.style.setProperty("--color-accent-border", "rgba(245,183,147,0.25)");
      root.style.setProperty("--color-border-accent", "rgba(245,183,147,0.20)");
      root.style.setProperty("--color-accent-text", "#1f2937");
      break;

    case "white":
      root.style.setProperty("--color-accent", "#e1e3e6");
      root.style.setProperty("--color-accent-hover", "#eceef0");
      root.style.setProperty("--color-accent-soft", "rgba(225,227,230,0.10)");
      root.style.setProperty("--color-accent-border", "rgba(225,227,230,0.25)");
      root.style.setProperty("--color-border-accent", "rgba(225,227,230,0.20)");
      root.style.setProperty("--color-accent-text", "#111827");
      break;

    default:
      root.style.setProperty("--color-accent", "#9b7cf0");
      root.style.setProperty("--color-accent-hover", "#b594fa");
      root.style.setProperty("--color-accent-soft", "rgba(155,124,240,0.10)");
      root.style.setProperty("--color-accent-border", "rgba(155,124,240,0.25)");
      root.style.setProperty("--color-border-accent", "rgba(155,124,240,0.20)");
      root.style.setProperty("--color-accent-text", "#ffffff");
  }
}
