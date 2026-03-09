export type VisualTier = 'off' | 'soft' | 'full';

function prefersReducedMotion(): boolean {
	if (typeof window === 'undefined' || typeof window.matchMedia === 'undefined') return false;
	return window.matchMedia('(prefers-reduced-motion: reduce)').matches;
}

function isLowPowerDevice(): boolean {
	if (typeof navigator === 'undefined') return false;

	const cores = navigator.hardwareConcurrency ?? 8;
	const navWithMemory = navigator as Navigator & { deviceMemory?: number };
	const memory = navWithMemory.deviceMemory ?? 8;

	return cores <= 4 || memory <= 4;
}

export function getVisualTier(): VisualTier {
	if (typeof window === 'undefined') return 'soft';
	if (prefersReducedMotion()) return 'off';
	if (isLowPowerDevice()) return 'soft';
	return 'full';
}
