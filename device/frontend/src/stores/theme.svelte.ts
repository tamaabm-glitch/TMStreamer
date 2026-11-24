
export type Theme = "auto" | "light" | "dark";

const KEY = "tmstreamer.theme";
const LEGACY_KEY = "ystreamer.theme";

function load(): Theme {
  try {
    const saved = localStorage.getItem(KEY) ?? localStorage.getItem(LEGACY_KEY);
    if (saved !== null) localStorage.setItem(KEY, saved);
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
        localStorage.removeItem(LEGACY_KEY);
      } else {
        localStorage.setItem(KEY, theme);
        localStorage.removeItem(LEGACY_KEY);
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
