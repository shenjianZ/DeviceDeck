const THEME_KEY = "dd-theme";

function isValidTheme(value: unknown): value is "dark" | "light" {
  return value === "dark" || value === "light";
}

export function getCachedTheme(): "dark" | "light" {
  const stored = localStorage.getItem(THEME_KEY);
  return isValidTheme(stored) ? stored : "dark";
}

export function initTheme(): void {
  const theme = getCachedTheme();
  document.documentElement.setAttribute("data-theme", theme);
}

export function applyTheme(theme: "dark" | "light") {
  document.documentElement.setAttribute("data-theme", theme);
  localStorage.setItem(THEME_KEY, theme);
}
