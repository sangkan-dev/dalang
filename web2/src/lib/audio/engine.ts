import { get } from 'svelte/store';
import { audioEnabled } from './store.js';

export type AudioIntent = 'hover' | 'click' | 'reveal' | 'section';

let audioCtx: AudioContext | null = null;
let initialized = false;

function canUseAudio(): boolean {
	return typeof window !== 'undefined' && typeof window.AudioContext !== 'undefined';
}

function ensureContext(): AudioContext | null {
	if (!canUseAudio()) return null;
	if (!audioCtx) {
		audioCtx = new window.AudioContext();
	}
	return audioCtx;
}

export async function unlockAudio(): Promise<void> {
	if (!get(audioEnabled)) return;
	const ctx = ensureContext();
	if (!ctx) return;
	if (ctx.state === 'suspended') {
		try {
			await ctx.resume();
			initialized = true;
		} catch {
			// Browser policy may block until explicit gesture; fail silently.
		}
	}
}

function envelope(gain: GainNode, now: number, durationMs: number): void {
	const end = now + durationMs / 1000;
	gain.gain.setValueAtTime(0.0001, now);
	gain.gain.exponentialRampToValueAtTime(0.18, now + 0.01);
	gain.gain.exponentialRampToValueAtTime(0.0001, end);
}

function intentProfile(intent: AudioIntent): {
	freq: number;
	durationMs: number;
	type: OscillatorType;
} {
	switch (intent) {
		case 'hover':
			return { freq: 540, durationMs: 70, type: 'triangle' };
		case 'click':
			return { freq: 390, durationMs: 95, type: 'sine' };
		case 'reveal':
			return { freq: 630, durationMs: 140, type: 'triangle' };
		case 'section':
			return { freq: 260, durationMs: 180, type: 'sawtooth' };
	}
}

export function playIntent(intent: AudioIntent): void {
	if (!get(audioEnabled)) return;
	const ctx = ensureContext();
	if (!ctx) return;
	if (ctx.state !== 'running' && !initialized) return;

	const profile = intentProfile(intent);
	const osc = ctx.createOscillator();
	const gain = ctx.createGain();
	const now = ctx.currentTime;

	osc.type = profile.type;
	osc.frequency.setValueAtTime(profile.freq, now);
	osc.connect(gain);
	gain.connect(ctx.destination);
	envelope(gain, now, profile.durationMs);

	osc.start(now);
	osc.stop(now + profile.durationMs / 1000);
}
