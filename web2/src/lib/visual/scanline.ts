import type { VisualTier } from './perf.js';

export type LayerOpacities = {
	holo: number;
	scanline: number;
	noise: number;
};

export function getLayerOpacities(tier: VisualTier): LayerOpacities {
	switch (tier) {
		case 'off':
			return { holo: 0, scanline: 0, noise: 0 };
		case 'soft':
			return { holo: 0.2, scanline: 0.12, noise: 0.08 };
		case 'full':
			return { holo: 0.34, scanline: 0.2, noise: 0.12 };
	}
}
