<script lang="ts">
	import type { VisualTier } from './perf.js';
	import { getLayerOpacities } from './scanline.js';

	type Props = {
		tier?: VisualTier;
	};

	const props: Props = $props();
	const tier = $derived(props.tier ?? 'soft');
	const opacity = $derived(getLayerOpacities(tier));
</script>

{#if tier !== 'off'}
	<div class="overlay-root" aria-hidden="true">
		<div class="overlay-holo" style={`opacity:${opacity.holo}`}></div>
		<div class="overlay-scanline" style={`opacity:${opacity.scanline}`}></div>
		<div class="overlay-noise" style={`opacity:${opacity.noise}`}></div>
	</div>
{/if}

<style>
	.overlay-root {
		pointer-events: none;
		position: fixed;
		inset: 0;
		z-index: 0;
		overflow: hidden;
	}

	.overlay-holo,
	.overlay-scanline,
	.overlay-noise {
		position: absolute;
		inset: 0;
	}

	.overlay-holo {
		background:
			radial-gradient(circle at 22% 18%, rgb(228 190 106 / 38%), transparent 42%),
			radial-gradient(circle at 70% 72%, rgb(221 107 32 / 24%), transparent 38%);
		animation: holo-drift 16s ease-in-out infinite alternate;
	}

	.overlay-scanline {
		background: repeating-linear-gradient(
			to bottom,
			rgb(255 255 255 / 0%) 0,
			rgb(255 255 255 / 0%) 2px,
			rgb(255 255 255 / 6%) 3px,
			rgb(255 255 255 / 0%) 4px
		);
		mix-blend-mode: soft-light;
		animation: scanline-shift 8s linear infinite;
	}

	.overlay-noise {
		background-image: radial-gradient(rgb(255 255 255 / 10%) 1px, transparent 1px);
		background-size: 3px 3px;
		mix-blend-mode: overlay;
		animation: noise-shift 0.36s steps(3, end) infinite;
	}

	@keyframes holo-drift {
		0% {
			transform: translate3d(-1%, -1%, 0) scale(1.02);
		}
		100% {
			transform: translate3d(1.2%, 1.4%, 0) scale(1.05);
		}
	}

	@keyframes scanline-shift {
		from {
			transform: translateY(-8px);
		}
		to {
			transform: translateY(8px);
		}
	}

	@keyframes noise-shift {
		0% {
			transform: translate(0, 0);
		}
		50% {
			transform: translate(1px, 1px);
		}
		100% {
			transform: translate(-1px, -1px);
		}
	}

	@media (prefers-reduced-motion: reduce) {
		.overlay-holo,
		.overlay-scanline,
		.overlay-noise {
			animation: none !important;
		}
	}
</style>
