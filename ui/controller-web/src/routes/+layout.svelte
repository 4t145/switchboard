<script lang="ts">
	import '../app.css';
	import { defaultThemeContext, useTheme } from '$lib/themes/useTheme';
	import { setContext } from 'svelte';
	let { children } = $props();
	let themeContext = defaultThemeContext();
	setContext('theme', themeContext);
</script>

{@render children()}
