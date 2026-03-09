import { writable } from 'svelte/store';

const STORAGE_KEY = 'dalang-web2-audio-enabled';

export const audioEnabled = writable(true);

export function initAudioPreferences(): void {
	if (typeof window === 'undefined') return;

	const raw = window.localStorage.getItem(STORAGE_KEY);
	if (raw === 'false') {
		audioEnabled.set(false);
		return;
	}

	audioEnabled.set(true);
}

export function setAudioEnabled(enabled: boolean): void {
	audioEnabled.set(enabled);
	if (typeof window === 'undefined') return;
	window.localStorage.setItem(STORAGE_KEY, String(enabled));
}

export function toggleAudio(): void {
	audioEnabled.update((current) => {
		const next = !current;
		if (typeof window !== 'undefined') {
			window.localStorage.setItem(STORAGE_KEY, String(next));
		}
		return next;
	});
}
