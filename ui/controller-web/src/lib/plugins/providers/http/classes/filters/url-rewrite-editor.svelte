<script lang="ts">
	import type { HttpClassEditorProps } from '$lib/plugins/types';

	type Props = HttpClassEditorProps;

	let { value = $bindable(), readonly = false }: Props = $props();

	// Initialize structure
	$effect(() => {
		if (!value || typeof value !== 'object') {
			value = { path: '' };
		}
		if (!value.path) {
			value.path = '';
		}
	});
</script>

<div class="space-y-3">
	<label class="label">
		<span class="label-text text-sm font-semibold">Rewrite Path</span>
		<input
			type="text"
			class="input input-sm"
			bind:value={value.path}
			{readonly}
			placeholder="/new/path or /{'{'}capture{'}'}"
		/>
		<span class="label-text-alt">
			Use placeholders like <code class="code text-xs">{'{'} rest {'}'}</code> to capture path segments
		</span>
	</label>

	<div class="card preset-outlined-surface p-3 text-xs space-y-1">
		<p class="font-semibold">Examples:</p>
		<ul class="list-disc list-inside opacity-75 space-y-1">
			<li><code class="code">/api</code> - Rewrite to fixed path</li>
			<li><code class="code">/{'{'} rest {'}'}</code> - Capture all segments</li>
			<li><code class="code">/v2/{'{'} path {'}'}</code> - Add prefix and capture</li>
		</ul>
	</div>
</div>
