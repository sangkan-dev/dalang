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
</svelte:head>

{@render children()}
