type ShortcutTarget = Pick<Window, "addEventListener" | "removeEventListener">;

const FUNCTION_KEY_RE = /^F(?:[1-9]|1[0-2])$/;
const APP_SHORTCUT_SCOPE = "[data-dd-app-shortcuts]";
const APP_CONTEXT_MENU_SCOPE = "[data-dd-context-menu]";

export function isShortcutKeyEvent(
  event: Pick<KeyboardEvent, "altKey" | "ctrlKey" | "key" | "metaKey">
) {
  return event.ctrlKey || event.metaKey || event.altKey || FUNCTION_KEY_RE.test(event.key);
}

export function blockShortcutKeyDown(event: KeyboardEvent) {
  if (!isShortcutKeyEvent(event)) return;
  if (shouldAllowAppShortcut(event)) return;

  event.preventDefault();
  event.stopPropagation();
  event.stopImmediatePropagation();
}

export function installShortcutBlocker(target: ShortcutTarget = window) {
  target.addEventListener("keydown", blockShortcutKeyDown, { capture: true });

  return () => {
    target.removeEventListener("keydown", blockShortcutKeyDown, { capture: true });
  };
}

export function blockContextMenu(event: MouseEvent) {
  if (isInsideElementScope(event.target, APP_CONTEXT_MENU_SCOPE)) return;

  event.preventDefault();
  event.stopPropagation();
  event.stopImmediatePropagation();
}

export function installContextMenuBlocker(
  target: ShortcutTarget = window,
  enabled = import.meta.env.PROD
) {
  if (!enabled) return () => {};

  target.addEventListener("contextmenu", blockContextMenu, { capture: true });

  return () => {
    target.removeEventListener("contextmenu", blockContextMenu, { capture: true });
  };
}

function shouldAllowAppShortcut(event: KeyboardEvent): boolean {
  if (isEditableTarget(event.target)) return true;

  const isSelectAll =
    (event.ctrlKey || event.metaKey) &&
    !event.altKey &&
    event.key.toLowerCase() === "a";

  if (!isSelectAll) return false;

  return document.querySelector(APP_SHORTCUT_SCOPE) !== null;
}

function isInsideElementScope(target: EventTarget | null, selector: string): boolean {
  return target instanceof HTMLElement && target.closest(selector) !== null;
}

function isEditableTarget(target: EventTarget | null): boolean {
  if (!(target instanceof HTMLElement)) return false;
  if (target.isContentEditable) return true;
  return ["INPUT", "TEXTAREA", "SELECT"].includes(target.tagName);
}
