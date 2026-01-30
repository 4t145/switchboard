<script lang="ts">
	import type { HttpClassEditorProps } from '$lib/plugins/types';

	export type UrlRewriteFilterConfig = {
		path: string | null;
		hostname: string | null;
	};
	type Props = HttpClassEditorProps<UrlRewriteFilterConfig>;
	let { value = $bindable(), readonly = false }: Props = $props();

	// Initialize structure
	$effect(() => {
		if (!value || typeof value !== 'object') {
			value = { path: null, hostname: null };
		}
		if (!value.path) {
			value.path = null;
		}
		if (!value.hostname) {
			value.hostname = null;
		}
	});
</script>

<div class="space-y-3">
	<label class="label">
		<span class="label-text text-sm font-semibold">Rewrite Path</span>
		<input
			type="text"
			class="input-sm input"
			bind:value={value.path}
			{readonly}
			placeholder="/new/path or /{'{'}capture{'}'}"
		/>
	</label>
	<label class="label">
		<span class="label-text text-sm font-semibold">Rewrite Host</span>
		<input
			type="text"
			class="input-sm input"
			bind:value={value.hostname}
			{readonly}
			placeholder="new.host.com"
		/>
	</label>
	<div class="preset-outlined-surface space-y-1 card p-3 text-xs">
		<p class="font-semibold">Examples:</p>
		<ul class="list-inside list-disc space-y-1 opacity-75">
			<li><code class="code">/api</code> - Rewrite to fixed path</li>
			<li>
				<code class="code">/{'{'}*rest{'}'}</code> or <code class="code">/*</code> - Capture all segments
			</li>
			<li><code class="code">/v2/{'{'}path{'}'}</code> - Add prefix and capture</li>
		</ul>
	</div>
</div>
