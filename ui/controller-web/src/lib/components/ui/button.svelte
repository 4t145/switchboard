<script lang="ts" module>
	import { getThemeClass, useTheme } from '$lib/themes/useTheme';
	import { Button } from 'bits-ui';
	import type { ButtonRootProps } from 'bits-ui';
	type Props = ButtonRootProps & {};
</script>

<script lang="ts">
	const props: Props = $props();
	const themeContext = useTheme();
</script>

<Button.Root {...props} class="{props.class} {getThemeClass(themeContext, 'Button')}">
	{#if props.children !== undefined}
		{@render props.children()}
	{/if}
</Button.Root>
