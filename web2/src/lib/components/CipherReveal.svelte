<script lang="ts">
	import { onMount } from 'svelte';

	type Props = {
		javaneseText?: string;
		latinText: string;
		durationMs?: number;
		delayMs?: number;
		startOnView?: boolean;
		className?: string;
		onDone?: (value: string) => void;
	};

	const props: Props = $props();

	const scrambleAlphabet = 'ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789';
	let host: HTMLSpanElement | null = null;
	let displayText = $state('ꦱꦁꦏꦤ꧀');
	let started = false;
	let completed = $state(false);

	$effect(() => {
		if (!started) {
			displayText = props.javaneseText ?? 'ꦱꦁꦏꦤ꧀';
		}
	});

	function reducedMotion(): boolean {
		if (typeof window === 'undefined' || typeof window.matchMedia === 'undefined') return false;
		return window.matchMedia('(prefers-reduced-motion: reduce)').matches;
	}

	function scrambleFor(progress: number): string {
		const latinText = props.latinText;
		const stableCount = Math.floor(progress * latinText.length);
		let out = '';
		for (let i = 0; i < latinText.length; i += 1) {
			if (i < stableCount) {
				out += latinText[i];
			} else {
				out += scrambleAlphabet[Math.floor(Math.random() * scrambleAlphabet.length)] ?? 'X';
			}
		}
		return out;
	}

	function runReveal(): void {
		if (started) return;
		started = true;
		const latinText = props.latinText;
		const durationMs = props.durationMs ?? 1400;
		const delayMs = props.delayMs ?? 0;

		if (reducedMotion()) {
			displayText = latinText;
			completed = true;
			props.onDone?.(latinText);
			return;
		}

		const start = performance.now() + delayMs;
		const tick = (now: number): void => {
			if (now < start) {
				requestAnimationFrame(tick);
				return;
			}

			const progress = Math.min((now - start) / durationMs, 1);
			displayText = progress >= 1 ? latinText : scrambleFor(progress);
			if (progress >= 1) {
				completed = true;
				props.onDone?.(latinText);
				return;
			}
			requestAnimationFrame(tick);
		};

		requestAnimationFrame(tick);
	}

	onMount(() => {
		if (!(props.startOnView ?? true)) {
			runReveal();
			return;
		}

		if (typeof window === 'undefined' || typeof IntersectionObserver === 'undefined') {
			runReveal();
			return;
		}

		const observer = new IntersectionObserver(
			(entries) => {
				for (const entry of entries) {
					if (entry.isIntersecting) {
						runReveal();
						observer.disconnect();
					}
				}
			},
			{ threshold: 0.5 }
		);

		if (host) observer.observe(host);
		return () => observer.disconnect();
	});
</script>

<span
	bind:this={host}
	class={`cipher-reveal ${completed ? 'is-complete text-data' : 'text-javanese'} ${props.className ?? ''}`}
	>{displayText}</span
>

<style>
	.cipher-reveal {
		display: inline-block;
		letter-spacing: 0.06em;
		white-space: nowrap;
	}

	.cipher-reveal.is-complete {
		letter-spacing: 0.16em;
		text-transform: uppercase;
	}
</style>
