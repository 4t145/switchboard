import type { PageLoad } from './$types';
import type { PresetThemeName } from '$lib/stores/settings.svelte';

// All themes are already imported in app.css via @import statements
// This list should match the themes imported in app.css
const AVAILABLE_THEMES: PresetThemeName[] = [
	'vintage',
	'modern',
	'rocket',
	'seafoam',
	'cerberus',
	'sahara',
	'astro'
];

export const load: PageLoad = async () => {
	return {
		themes: AVAILABLE_THEMES
	};
};
