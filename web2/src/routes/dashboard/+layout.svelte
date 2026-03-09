<script lang="ts">
	const { children } = $props();
</script>

<div class="dashboard-shell">
	<header class="dashboard-header">
		<p class="dashboard-kicker">DALANG DASHBOARD</p>
		<h1 class="dashboard-title">Operational Console</h1>
	</header>
	<main class="dashboard-main">{@render children()}</main>
</div>
