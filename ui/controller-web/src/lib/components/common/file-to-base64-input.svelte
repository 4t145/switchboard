<script lang="ts">
	import { FileUpload } from '@skeletonlabs/skeleton-svelte';
	import { Upload, X, FileText, FileIcon } from 'lucide-svelte';

	let {
		value = $bindable(''),
		label = 'Upload File',
		accept = '*',
		helperText = ''
	} = $props<{
		value: string;
		label?: string;
		accept?: string;
		helperText?: string;
	}>();

	let fileName = $state<string | null>(null);

	// Simple heuristic to detect if current value is likely Base64 content vs empty
	$effect(() => {
		if (value && !fileName) {
			fileName = 'Existing Content (Base64)';
		} else if (!value) {
			fileName = null;
		}
	});

	function handleFileSelect(details: { acceptedFiles: File[] }) {
		const file = details.acceptedFiles[0];
		if (!file) return;

		fileName = file.name;
		const reader = new FileReader();
		reader.onload = (e) => {
			const result = e.target?.result;
			if (typeof result === 'string') {
				// Extract base64 part if it's a data URL, or just take the content?
				// FileReader.readAsDataURL returns "data:application/octet-stream;base64,....."
				// We usually just want the base64 part.
				const base64 = result.split(',')[1];
				value = base64;
			}
		};
		reader.readAsDataURL(file);
	}

	function clear() {
		value = '';
		fileName = null;
	}
</script>

<div class="form-control w-full max-w-xs">
	<label class="label">
		<span class="label-text">{label}</span>
	</label>

	{#if !value}
		<FileUpload {accept} onFileChange={handleFileSelect} class="w-full">
			{#if helperText}
				<FileUpload.Label>
					{helperText}
				</FileUpload.Label>
			{/if}
			<FileUpload.Dropzone>
				<FileIcon class="size-10" />
				<FileUpload.Trigger>Browse Files</FileUpload.Trigger>
				<FileUpload.HiddenInput />
			</FileUpload.Dropzone>
			<FileUpload.ItemGroup>
				<FileUpload.Context>
					{#snippet children(fileUpload)}
						{#each fileUpload().acceptedFiles as file (file.name)}
							<FileUpload.Item {file}>
								<FileUpload.ItemName>{file.name}</FileUpload.ItemName>
								<FileUpload.ItemSizeText>{file.size} bytes</FileUpload.ItemSizeText>
								<FileUpload.ItemDeleteTrigger />
							</FileUpload.Item>
						{/each}
					{/snippet}
				</FileUpload.Context>
			</FileUpload.ItemGroup>
			<FileUpload.ClearTrigger>Clear Files</FileUpload.ClearTrigger>
		</FileUpload>
	{:else}
		<div
			class="flex items-center gap-2 rounded-lg border border-surface-300 bg-surface-50 p-2 dark:border-surface-600 dark:bg-surface-800"
		>
			<FileText size={20} class="text-primary-500" />

			<span class="flex-1 truncate text-sm font-medium">{fileName}</span>
			<button class="hover:preset-tonal-error btn-icon btn-icon-sm" onclick={clear} title="Clear">
				<X size={16} />
			</button>
		</div>
	{/if}
</div>
