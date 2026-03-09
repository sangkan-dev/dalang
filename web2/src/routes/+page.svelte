<script lang="ts">
	import { onMount } from 'svelte';
	import { resolve } from '$app/paths';
	import CipherReveal from '$lib/components/CipherReveal.svelte';
	import OverlayLayer from '$lib/visual/OverlayLayer.svelte';
	import { getVisualTier } from '$lib/visual/perf.js';
	import type { VisualTier } from '$lib/visual/perf.js';
	import {
		audioEnabled,
		initAudioPreferences,
		toggleAudio,
		unlockAudio,
		playIntent
	} from '$lib/audio/index.js';

	let visualTier = $state<VisualTier>('soft');
	let cipherDone = $state(false);

	onMount(() => {
		initAudioPreferences();
		visualTier = getVisualTier();
	});

	async function prepareInteraction(): Promise<void> {
		await unlockAudio();
	}

	async function onActionClick(): Promise<void> {
		await prepareInteraction();
		playIntent('click');
	}

	function onActionHover(): void {
		playIntent('hover');
	}

	async function onToggleAudio(): Promise<void> {
		await prepareInteraction();
		toggleAudio();
		playIntent('click');
	}

	function onCipherDone(): void {
		cipherDone = true;
		playIntent('reveal');
	}

	type CinematicParams = {
		delay?: number;
	};

	function cinematic(node: HTMLElement, params: CinematicParams = {}) {
		if (typeof window === 'undefined') return;

		node.style.setProperty('--reveal-delay', `${params.delay ?? 0}ms`);

		const reduced = window.matchMedia('(prefers-reduced-motion: reduce)').matches;
		if (reduced || typeof IntersectionObserver === 'undefined') {
			node.classList.add('is-visible');
			return;
		}

		if (node.getBoundingClientRect().top <= window.innerHeight * 0.96) {
			node.classList.add('is-visible');
			return;
		}

		const observer = new IntersectionObserver(
			(entries) => {
				for (const entry of entries) {
					if (entry.isIntersecting || entry.intersectionRatio > 0) {
						node.classList.add('is-visible');
						observer.disconnect();
					}
				}
			},
			{ threshold: [0, 0.12, 0.22], rootMargin: '0px 0px -6% 0px' }
		);

		observer.observe(node);
		return {
			destroy() {
				observer.disconnect();
			}
		};
	}
</script>

<svelte:head>
	<title>Dalang | Ancient Cybernetics Security Operator</title>
	<meta
		name="description"
		content="Dalang by Sangkan is an autonomous offensive security operator with browser-native execution, live event streams, and reproducible audit reports."
	/>
	<meta
		name="keywords"
		content="Dalang, Sangkan, offensive security, cybersecurity automation, browser security testing, autonomous pentest"
	/>
	<meta name="robots" content="index,follow,max-image-preview:large" />
	<link rel="canonical" href="https://sangkan.dev" />
	<meta property="og:type" content="website" />
	<meta property="og:site_name" content="Sangkan" />
	<meta property="og:title" content="Dalang | Ancient Cybernetics Security Operator" />
	<meta
		property="og:description"
		content="Autonomous security operations with browser-native tooling, cinematic control flow, and report-grade outputs."
	/>
	<meta property="og:url" content="https://sangkan.dev" />
	<meta name="twitter:card" content="summary_large_image" />
	<meta name="twitter:title" content="Dalang | Ancient Cybernetics Security Operator" />
	<meta
		name="twitter:description"
		content="Autonomous security operations with browser-native tooling and reproducible reporting."
	/>
</svelte:head>

<OverlayLayer tier={visualTier} />

<main class="landing-root relative z-[1] pb-14 md:pb-20">
	<section
		use:cinematic={{ delay: 40 }}
		class="landing-shell cinematic-pro px-5 pt-12 pb-14 md:pt-24 md:pb-[4.5rem]"
		aria-labelledby="landing-title"
	>
		<div class="flex items-center justify-between gap-4 max-sm:flex-col max-sm:items-start">
			<p class="m-0 font-mono text-[0.78rem] tracking-[0.14em] text-[var(--color-ash)] uppercase">
				SANGKAN PRESENTS
			</p>
			<button
				class="inline-flex cursor-pointer items-center justify-center rounded-[var(--radius-md)] border border-[rgb(228_190_106_/_35%)] bg-[linear-gradient(160deg,rgb(255_255_255_/_3%),rgb(255_255_255_/_1%))] px-4 py-2 font-mono text-[0.75rem] font-semibold tracking-[0.08em] text-[var(--color-gold-bright)] uppercase shadow-[var(--shadow-hairline)] transition duration-150 ease-out hover:-translate-y-px hover:border-[rgb(228_190_106_/_55%)] hover:brightness-105"
				type="button"
				onclick={onToggleAudio}
			>
				{#if $audioEnabled}
					Sound: On
				{:else}
					Sound: Off
				{/if}
			</button>
		</div>

		<h1
			id="landing-title"
			class="mt-1.5 mb-0 text-[clamp(2.5rem,8vw,5rem)] leading-[0.94] text-[var(--color-gold-bright)] [text-shadow:var(--shadow-gold-soft)]"
		>
			DALANG
		</h1>
		<p
			class="mt-3 mb-6 font-mono text-[0.95rem] tracking-[0.1em] text-[var(--color-ash)] uppercase"
		>
			Ancient Cybernetics Security Operator
		</p>

		<p class="m-0 max-w-[64ch] text-[var(--color-base-text)]">
			Offensive security operations should feel deliberate, traceable, and fast. Dalang blends
			autonomous reasoning, browser-level execution, and reproducible reporting into one cybernetic
			workflow.
		</p>

		<p
			class="m-0 inline-flex items-center gap-3 pt-6 text-[0.95rem] text-[var(--color-gold-bright)]"
			aria-live="polite"
		>
			<CipherReveal latinText="SANGKAN" delayMs={220} onDone={onCipherDone} />
			{#if cipherDone}
				<span
					class="rounded-[var(--radius-pill)] bg-[linear-gradient(120deg,var(--color-gold),var(--color-gold-bright))] px-2 py-1 font-mono text-[0.72rem] tracking-[0.09em] text-[#1d1200] uppercase"
					>Identity Synced</span
				>
			{/if}
		</p>

		<div class="mt-6 flex gap-3 max-sm:flex-col max-sm:items-stretch">
			<a
				class="inline-flex items-center justify-center rounded-[var(--radius-md)] bg-[linear-gradient(135deg,var(--color-gold),var(--color-gold-bright))] px-4 py-2.5 font-semibold !text-[#1a1406] no-underline shadow-[var(--shadow-hairline),var(--shadow-gold-soft)] transition duration-150 ease-out hover:-translate-y-px hover:brightness-105"
				href={resolve('/dashboard')}
				onclick={onActionClick}
				onmouseenter={onActionHover}>Enter Dashboard</a
			>
			<a
				class="inline-flex items-center justify-center rounded-[var(--radius-md)] border border-[rgb(228_190_106_/_35%)] bg-[linear-gradient(160deg,rgb(255_255_255_/_3%),rgb(255_255_255_/_1%))] px-4 py-2.5 font-semibold text-[var(--color-gold-bright)] no-underline shadow-[var(--shadow-hairline)] transition duration-150 ease-out hover:-translate-y-px hover:border-[rgb(228_190_106_/_55%)] hover:brightness-105"
				href={resolve('/dashboard/skills')}
				onclick={onActionClick}
				onmouseenter={onActionHover}>Inspect API Surface</a
			>
		</div>
	</section>

	<section
		use:cinematic={{ delay: 120 }}
		class="landing-shell cinematic-pro px-5 pt-3 pb-0"
		aria-labelledby="problem-title"
	>
		<h2
			id="problem-title"
			class="mt-0 mb-4 text-[clamp(1.35rem,3.8vw,2.1rem)] leading-[1.15] text-[var(--color-gold-bright)]"
		>
			Built For Real Audit Pressure
		</h2>
		<div class="pressure-board surface-panel p-6 md:p-8">
			<div class="relative z-[1] space-y-3">
				<p
					class="m-0 font-mono text-[0.72rem] tracking-[0.14em] text-[var(--color-gold-bright)] uppercase"
				>
					OPERATOR SIGNAL
				</p>
				<p class="m-0 max-w-[64ch] text-[var(--color-base-text)]">
					Dalang is engineered for moments when noise is high, time is short, and every decision
					must stay traceable. You do not lose context while the target shifts.
				</p>
			</div>
			<div
				class="relative z-[1] mt-6 grid gap-4 md:grid-cols-2 xl:grid-cols-3"
				role="list"
				aria-label="Operational metrics"
			>
				<div class="metric-cell" role="listitem">
					<p
						class="m-0 font-mono text-[clamp(1.3rem,4vw,2rem)] tracking-[0.06em] text-[var(--color-gold-bright)]"
					>
						35+
					</p>
					<p class="mt-2 text-[0.88rem] text-[var(--color-ash)]">
						Browser primitives for offensive workflows
					</p>
				</div>
				<div class="metric-cell" role="listitem">
					<p
						class="m-0 font-mono text-[clamp(1.3rem,4vw,2rem)] tracking-[0.06em] text-[var(--color-gold-bright)]"
					>
						22
					</p>
					<p class="mt-2 text-[0.88rem] text-[var(--color-ash)]">
						Bundled skills across web, cloud, network, and container
					</p>
				</div>
				<div class="metric-cell" role="listitem">
					<p
						class="m-0 font-mono text-[clamp(1.3rem,4vw,2rem)] tracking-[0.06em] text-[var(--color-gold-bright)]"
					>
						Live
					</p>
					<p class="mt-2 text-[0.88rem] text-[var(--color-ash)]">
						WebSocket event stream with replayable sessions
					</p>
				</div>
			</div>
		</div>
	</section>

	<section
		use:cinematic={{ delay: 170 }}
		class="landing-shell cinematic-pro px-5 pt-6 pb-0"
		aria-labelledby="arsenal-title"
	>
		<p class="signal-line" aria-hidden="true">
			AUTONOMOUS FLOW / TRACE / EXECUTE / OBSERVE / REPORT
		</p>
		<h2
			id="arsenal-title"
			class="mt-0 mb-4 text-[clamp(1.35rem,3.8vw,2.1rem)] leading-[1.15] text-[var(--color-gold-bright)]"
		>
			Operational Flow, Cinematic by Design
		</h2>
		<p class="m-0 max-w-[62ch] text-[var(--color-ash)]">
			Instead of juggling disconnected tools, Dalang keeps one continuous offensive rhythm from
			first contact to final report.
		</p>
		<ol class="mt-6 grid gap-4" aria-label="Operational flow stages">
			<li class="flow-item">
				<span class="flow-index">01</span>
				<div>
					<h3 class="m-0 text-[1.05rem] text-[var(--color-base-text)]">
						Map attack surface in motion
					</h3>
					<p class="mt-1.5 text-[var(--color-ash)]">
						Chain reconnaissance paths dynamically as new evidence appears.
					</p>
				</div>
			</li>
			<li class="flow-item">
				<span class="flow-index">02</span>
				<div>
					<h3 class="m-0 text-[1.05rem] text-[var(--color-base-text)]">
						Drive browser-native actions
					</h3>
					<p class="mt-1.5 text-[var(--color-ash)]">
						Use CDP-level controls to operate SPA targets with repeatable intent.
					</p>
				</div>
			</li>
			<li class="flow-item">
				<span class="flow-index">03</span>
				<div>
					<h3 class="m-0 text-[1.05rem] text-[var(--color-base-text)]">
						Lock evidence into report-grade output
					</h3>
					<p class="mt-1.5 text-[var(--color-ash)]">
						Preserve timeline, findings, and context for engineering handover.
					</p>
				</div>
			</li>
		</ol>
	</section>

	<section
		use:cinematic={{ delay: 220 }}
		class="landing-shell cinematic-pro px-5 pt-6 pb-0"
		aria-labelledby="start-title"
	>
		<h2
			id="start-title"
			class="mt-0 mb-4 text-[clamp(1.35rem,3.8vw,2.1rem)] leading-[1.15] text-[var(--color-gold-bright)]"
		>
			Launch Sequence In Under 3 Minutes
		</h2>
		<div class="surface-panel terminal-stage space-y-4 p-6 md:p-8">
			<p class="m-0 font-mono text-[0.72rem] tracking-[0.14em] text-[var(--color-gold)] uppercase">
				SANGKAN BOOT SEQUENCE
			</p>
			<pre class="terminal-block"><code
					>$ dalang init
$ dalang login --provider copilot
$ dalang web --port 8080 --open</code
				></pre>
			<p class="m-0 text-[var(--color-base-text)]">
				One sequence. Full control room. Zero context loss between steps.
			</p>
		</div>
	</section>

	<section
		use:cinematic={{ delay: 270 }}
		class="landing-shell cinematic-pro px-5 pt-6 pb-0"
		aria-labelledby="cta-title"
	>
		<div class="surface-panel final-cta space-y-4 p-6 md:p-8">
			<p
				class="m-0 font-mono text-[0.72rem] tracking-[0.14em] text-[var(--color-gold-bright)] uppercase"
			>
				SANGKAN EXECUTION PROTOCOL
			</p>
			<h2
				id="cta-title"
				class="mt-0 mb-0 text-[clamp(1.35rem,3.8vw,2.1rem)] leading-[1.15] text-[var(--color-gold-bright)]"
			>
				From first probe to final report, keep momentum intact.
			</h2>
			<p class="m-0 max-w-[66ch] text-[var(--color-base-text)]">
				Operate with autonomous branching, browser-native control, and reproducible output that
				engineering teams can immediately act on.
			</p>
			<div class="mt-2 flex gap-3 max-sm:flex-col max-sm:items-stretch">
				<a
					class="inline-flex items-center justify-center rounded-[var(--radius-md)] bg-[linear-gradient(135deg,var(--color-gold),var(--color-gold-bright))] px-4 py-2.5 font-semibold !text-[#1a1406] no-underline shadow-[var(--shadow-hairline),var(--shadow-gold-soft)] transition duration-150 ease-out hover:-translate-y-px hover:brightness-105"
					href={resolve('/dashboard')}
					onclick={onActionClick}
					onmouseenter={onActionHover}>Open Control Room</a
				>
				<a
					class="inline-flex items-center justify-center rounded-[var(--radius-md)] border border-[rgb(228_190_106_/_35%)] bg-[linear-gradient(160deg,rgb(255_255_255_/_3%),rgb(255_255_255_/_1%))] px-4 py-2.5 font-semibold text-[var(--color-gold-bright)] no-underline shadow-[var(--shadow-hairline)] transition duration-150 ease-out hover:-translate-y-px hover:border-[rgb(228_190_106_/_55%)] hover:brightness-105"
					href="https://github.com/sangkan-dev/dalang"
					target="_blank"
					rel="noreferrer"
					onclick={onActionClick}
					onmouseenter={onActionHover}>Browse Repository</a
				>
			</div>
		</div>
	</section>

	<footer
		class="landing-shell mt-10 flex flex-wrap items-center justify-between gap-4 border-t border-[rgb(255_255_255_/_10%)] px-5 pt-8 pb-10"
		aria-label="Dalang footer"
	>
		<p class="m-0 font-mono text-[0.7rem] tracking-[0.14em] text-[var(--color-ash)] uppercase">
			DALANG / ANCIENT CYBERNETICS SECURITY OPERATOR
		</p>
		<div
			class="flex flex-wrap gap-4 font-mono text-[0.78rem] tracking-[0.06em] text-[var(--color-gold-bright)] uppercase"
		>
			<a class="hover:brightness-110" href={resolve('/dashboard')} onclick={onActionClick}
				>Dashboard</a
			>
			<a
				class="hover:brightness-110"
				href="https://docs-dalang.sangkan.dev"
				target="_blank"
				rel="noreferrer"
				onclick={onActionClick}>Docs</a
			>
			<a class="hover:brightness-110" href={resolve('/dashboard/skills')} onclick={onActionClick}
				>API</a
			>
			<a
				class="hover:brightness-110"
				href="https://github.com/sangkan-dev/dalang"
				target="_blank"
				rel="noreferrer"
				onclick={onActionClick}>GitHub</a
			>
			<a
				class="hover:brightness-110"
				href="https://sangkan.dev"
				target="_blank"
				rel="noreferrer"
				onclick={onActionClick}>sangkan.dev</a
			>
		</div>
	</footer>
</main>
