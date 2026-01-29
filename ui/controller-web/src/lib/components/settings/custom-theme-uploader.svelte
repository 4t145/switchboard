<script lang="ts">
	import { FileUpload } from '@skeletonlabs/skeleton-svelte';
	import { Upload, X, CheckCircle, AlertCircle } from 'lucide-svelte';
	import { customThemesStore } from '$lib/stores/custom-themes.svelte';
	import { m } from '$lib/paraglide/messages';
	
	const msg = m as any;

	let uploading = $state(false);
	let uploadError = $state<string | null>(null);
	let uploadSuccess = $state(false);

	async function handleFileSelect(details: { acceptedFiles: File[] }) {
		const file = details.acceptedFiles[0];
		if (!file) return;

		// Validate file type
		if (!file.name.toLowerCase().endsWith('.css')) {
			uploadError = msg.settings_custom_themes_file_only_css();
			return;
		}

		// Validate file size (500KB)
		if (file.size > 500 * 1024) {
			uploadError = msg.settings_custom_themes_file_too_large();
			return;
		}

		// Check if can add more themes
		if (!customThemesStore.canAddMore) {
			uploadError = msg.settings_custom_themes_limit_reached();
			return;
		}

		uploading = true;
		uploadError = null;
		uploadSuccess = false;

		try {
			const result = await customThemesStore.addTheme(file);
			
			if (result.success) {
				uploadSuccess = true;
				uploadError = null;
				
				// Auto-hide success message after 3 seconds
				setTimeout(() => {
					uploadSuccess = false;
				}, 3000);
			} else {
				uploadError = result.error || msg.settings_custom_themes_error();
			}
		} catch (error) {
			console.error('Upload error:', error);
			uploadError = msg.settings_custom_themes_error();
		} finally {
			uploading = false;
		}
	}

	function clearMessages() {
		uploadError = null;
		uploadSuccess = false;
	}
</script>

<div class="space-y-4">
	<div class="flex items-center justify-between">
		<h3 class="h3 text-lg font-semibold">{msg.settings_custom_themes_upload()}</h3>
		<a 
			href="https://themes.skeleton.dev" 
			target="_blank" 
			rel="noopener noreferrer"
			class="text-sm text-primary-600 hover:text-primary-700 hover:underline dark:text-primary-400"
		>
			{msg.settings_custom_themes_generator_link()} →
		</a>
	</div>

	<p class="text-sm opacity-75">
		{msg.settings_custom_themes_upload_desc()}
	</p>

	<!-- File Upload -->
	<FileUpload 
		accept=".css,text/css" 
		onFileChange={handleFileSelect}
		class="w-full"
		disabled={uploading || !customThemesStore.canAddMore}
	>
		<FileUpload.Dropzone class="border-2 border-dashed  rounded-lg p-8 text-center hover:border-primary-500 dark:hover:border-primary-400 transition-colors">
			<div class="flex flex-col items-center gap-3">
				<Upload size={32} class="opacity-50" />
				<div>
					<FileUpload.Trigger class="btn preset-filled-primary-500 btn-sm">
						{uploading ? 'Uploading...' : msg.settings_custom_themes_add()}
					</FileUpload.Trigger>
				</div>
				<p class="text-sm opacity-60">
					{msg.settings_custom_themes_drop_file()}
				</p>
				<p class="text-xs opacity-50">
					{customThemesStore.count} / 10 themes
				</p>
			</div>
			<FileUpload.HiddenInput />
		</FileUpload.Dropzone>

		<FileUpload.ItemGroup>
			<FileUpload.Context>
				{#snippet children(fileUpload)}
					{#each fileUpload().acceptedFiles as file}
						<FileUpload.Item {file} class="flex items-center gap-2 p-2 bg-surface-100 dark:bg-surface-800 rounded mt-2">
							<FileUpload.ItemName class="flex-1 text-sm">{file.name}</FileUpload.ItemName>
							<FileUpload.ItemSizeText class="text-xs opacity-60">{Math.round(file.size / 1024)} KB</FileUpload.ItemSizeText>
							<FileUpload.ItemDeleteTrigger class="btn btn-sm preset-ghost-error">
								<X size={16} />
							</FileUpload.ItemDeleteTrigger>
						</FileUpload.Item>
					{/each}
				{/snippet}
			</FileUpload.Context>
		</FileUpload.ItemGroup>
	</FileUpload>

	<!-- Success Message -->
	{#if uploadSuccess}
		<div class="card preset-tonal-success p-4 flex items-center gap-3">
			<CheckCircle size={20} />
			<span class="flex-1">{msg.settings_custom_themes_success()}</span>
			<button onclick={clearMessages} class="btn btn-sm preset-ghost">
				<X size={16} />
			</button>
		</div>
	{/if}

	<!-- Error Message -->
	{#if uploadError}
		<div class="card preset-tonal-error p-4 flex items-center gap-3">
			<AlertCircle size={20} />
			<span class="flex-1">{uploadError}</span>
			<button onclick={clearMessages} class="btn btn-sm preset-ghost">
				<X size={16} />
			</button>
		</div>
	{/if}
</div>
