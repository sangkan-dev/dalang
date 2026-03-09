<script lang="ts">
	import './layout.css';

	const { children } = $props();
</script>

<svelte:head>
	<title>Dalang | Sangkan</title>
	<meta
		name="description"
		content="Dalang is a cybersecurity AI framework by Sangkan for autonomous auditing and offensive security workflows."
	/>
	<meta name="robots" content="index,follow,max-image-preview:large" />
	<meta property="og:site_name" content="Sangkan" />
	<meta property="og:type" content="website" />
	<meta property="og:title" content="Dalang | Sangkan" />
	<meta
		property="og:description"
		content="Cybersecurity AI framework for autonomous audit operations, browser-native execution, and reproducible reporting."
	/>
	<meta name="twitter:card" content="summary_large_image" />
	<meta name="twitter:title" content="Dalang | Sangkan" />
	<meta
		name="twitter:description"
		content="Cybersecurity AI framework for autonomous audit operations and reproducible reporting."
	/>
</svelte:head>

{@render children()}
