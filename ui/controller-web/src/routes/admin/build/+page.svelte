<script lang="ts">
	import BuildWizard from '$lib/components/build/build-wizard.svelte';
</script>

<svelte:head>
	<title>Configuration Builder</title>
</svelte:head>

<div class="flex h-full flex-col">
	<!-- Clean full-screen container, passing height control to wizard -->
	<BuildWizard />
</div>
