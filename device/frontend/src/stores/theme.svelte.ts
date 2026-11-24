
import { readMigrated } from "@/lib/storage";

export type Theme = "auto" | "light" | "dark";

const KEY = "tmstreamer.theme";
const LEGACY_KEY = "ystreamer.theme";

function load(): Theme {
  try {
    const saved = readMigrated(KEY, LEGACY_KEY);
    return saved === "light" || saved === "dark" ? saved : "dark";
  } catch {
    return "dark";
  }
}

class ThemeStore {
  value = $state<Theme>(load());

  set(theme: Theme) {
    this.value = theme;

    try {
      if (theme === "auto") {
        localStorage.removeItem(KEY);
      } else {
        localStorage.setItem(KEY, theme);
      }
    } catch {}

    apply(theme);
  }
}

function apply(theme: Theme) {
  const root = document.documentElement;

  root.classList.toggle("light", theme === "light");
  root.classList.toggle("dark", theme === "dark");

  root.style.colorScheme = theme === "auto" ? "light dark" : theme;
}

export const theme = new ThemeStore();
