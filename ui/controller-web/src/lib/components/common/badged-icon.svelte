<script lang="ts">
	import type { Component, Snippet } from 'svelte';

	let {
		badge,
		children
	}: {
		badge: Snippet;
		children: Snippet;
	} = $props();
</script>

<div class="relative inline-flex items-center justify-center">
	<!-- Main Icon with Cutout Mask -->
	<div
		class="relative flex items-center justify-center"
		style="mask-image: radial-gradient(circle at 83% 83%, transparent 33%, black 35%); -webkit-mask-image: radial-gradient(circle at 83% 83%, transparent 33%, black 35%);"
	>
		{@render children()}
	</div>
	<!-- Badge: Positioned bottom-right -->
	<div
		class="absolute -right-[10%] -bottom-[10%] flex size-[55%] items-center justify-center rounded-full"
	>
		{@render badge()}
	</div>
</div>
