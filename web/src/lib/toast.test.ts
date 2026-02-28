import { describe, it, expect } from 'vitest';
import { addToast, removeToast, subscribe } from './toast.ts';

describe('toast store', () => {
  it('should add and remove toasts', () => {
    let toasts: Array<{ id: number; message: string }> = [];
    const unsub = subscribe((t) => { toasts = t; });

    addToast('Test message', 'info', 0); // no auto-dismiss
    expect(toasts).toHaveLength(1);
    expect(toasts[0].message).toBe('Test message');

    const id = toasts[0].id;
    removeToast(id);
    expect(toasts).toHaveLength(0);

    unsub();
  });

  it('should support multiple toasts', () => {
    let toasts: Array<{ id: number; message: string }> = [];
    const unsub = subscribe((t) => { toasts = t; });

    addToast('First', 'success', 0);
    addToast('Second', 'error', 0);
    expect(toasts).toHaveLength(2);

    // Cleanup
    for (const t of [...toasts]) removeToast(t.id);
    unsub();
  });

  it('should unsubscribe correctly', () => {
    let callCount = 0;
    const unsub = subscribe(() => { callCount++; });
    // Initial call
    expect(callCount).toBe(1);

    unsub();
    addToast('After unsub', 'info', 0);
    expect(callCount).toBe(1); // Should not increment

    // Cleanup
    let toasts: Array<{ id: number }> = [];
    const unsub2 = subscribe((t) => { toasts = t; });
    for (const t of [...toasts]) removeToast(t.id);
    unsub2();
  });
});
