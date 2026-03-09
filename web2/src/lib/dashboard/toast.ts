export type ToastType = 'success' | 'error' | 'warning' | 'info';

export interface ToastItem {
	id: number;
	message: string;
	type: ToastType;
	timeout: number;
}

let nextId = 0;
let toasts: ToastItem[] = [];
let listeners: Array<(items: ToastItem[]) => void> = [];

function notify(): void {
	for (const listener of listeners) {
		listener([...toasts]);
	}
}

export function subscribeToasts(listener: (items: ToastItem[]) => void): () => void {
	listeners = [...listeners, listener];
	listener([...toasts]);

	return () => {
		listeners = listeners.filter((entry) => entry !== listener);
	};
}

export function removeToast(id: number): void {
	toasts = toasts.filter((item) => item.id !== id);
	notify();
}

export function addToast(message: string, type: ToastType = 'info', timeout = 4000): void {
	const id = ++nextId;
	toasts = [...toasts, { id, message, type, timeout }];
	notify();

	if (timeout > 0) {
		setTimeout(() => removeToast(id), timeout);
	}
}

export const toast = {
	success: (message: string) => addToast(message, 'success'),
	error: (message: string) => addToast(message, 'error', 6000),
	warning: (message: string) => addToast(message, 'warning', 5000),
	info: (message: string) => addToast(message, 'info')
};
