/**
 * Global toast notification store using Svelte 5 runes.
 */

export type ToastType = 'success' | 'error' | 'warning' | 'info';

export interface Toast {
  id: number;
  message: string;
  type: ToastType;
  timeout: number;
}

let nextId = 0;

// We use a module-level reactive array via $state in the component.
// This module exports imperative add/remove functions and a getter.
let _toasts: Toast[] = [];
let _listeners: Array<(toasts: Toast[]) => void> = [];

function notify(): void {
  for (const fn of _listeners) fn([..._toasts]);
}

export function subscribe(fn: (toasts: Toast[]) => void): () => void {
  _listeners.push(fn);
  fn([..._toasts]);
  return () => {
    _listeners = _listeners.filter((l) => l !== fn);
  };
}

export function addToast(message: string, type: ToastType = 'info', timeout: number = 4000): void {
  const id = ++nextId;
  _toasts = [..._toasts, { id, message, type, timeout }];
  notify();

  if (timeout > 0) {
    setTimeout(() => removeToast(id), timeout);
  }
}

export function removeToast(id: number): void {
  _toasts = _toasts.filter((t) => t.id !== id);
  notify();
}

// Convenience shortcuts
export const toast = {
  success: (msg: string) => addToast(msg, 'success'),
  error: (msg: string) => addToast(msg, 'error', 6000),
  warning: (msg: string) => addToast(msg, 'warning', 5000),
  info: (msg: string) => addToast(msg, 'info'),
};
