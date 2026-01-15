<script lang="ts">
	import { Accordion } from '@skeletonlabs/skeleton-svelte';
	import { Upload, FileText, ChevronDown } from 'lucide-svelte';
	import { slide } from 'svelte/transition';

	let {
		value = $bindable(),
		label = 'PEM Certificate',
		helperText = ''
	} = $props<{
		value: string | string[];
		label?: string;
		helperText?: string;
	}>();

	// Convert base64 value(s) to PEM string for display
	function toPem(val: string | string[]): string {
		if (!val) return '';
		if (Array.isArray(val)) {
			return val.map((v) => safeAtob(v)).join('\n');
		}
		return safeAtob(val);
	}

	function safeAtob(str: string): string {
		try {
			return window.atob(str);
		} catch (e) {
			console.error('Invalid base64', e);
			return str;
		}
	}

	function safeBtoa(str: string): string {
		try {
			return window.btoa(str);
		} catch (e) {
			console.error('Encoding error', e);
			return str;
		}
	}

	// Internal text state
	let rawText = $state(toPem(value));

	// Watch for external value changes - but be careful about loops
	// We only want to update rawText if the value changed from OUTSIDE, 
	// not if we just updated it ourselves via updateValue.
	// However, simple comparison works if updateValue keeps consistency.
	$effect(() => {
		const currentPem = toPem(value);
		// If the external value (converted to PEM) is different from what we show, update showing
		// This handles switching between items where 'value' prop changes completely
		if (currentPem !== rawText) {
			rawText = currentPem;
		}
	});

	// Update value when text changes
	function handleInput(e: Event) {
		const target = e.target as HTMLTextAreaElement;
		rawText = target.value;
		updateValue(rawText);
	}

	function updateValue(text: string) {
		if (Array.isArray(value)) {
			// Extract multiple PEMs
			const pems = extractPems(text);
			value = pems.map(safeBtoa);
		} else {
			value = safeBtoa(text);
		}
	}

	function extractPems(text: string): string[] {
		// Regex to match PEM blocks: -----BEGIN ... ----- ... -----END ... -----
		const regex = /-----BEGIN [^-]+-----[\s\S]+?-----END [^-]+-----/g;
		const matches = text.match(regex);
		return matches || (text.trim() ? [text] : []);
	}

	function handleFileSelect(e: Event) {
		// Prevent the accordion from toggling when clicking the upload button
		e.stopPropagation();
		
		const input = e.target as HTMLInputElement;
		const file = input.files?.[0];
		if (!file) return;

		const reader = new FileReader();
		reader.onload = (e) => {
			const result = e.target?.result;
			if (typeof result === 'string') {
				rawText = result;
				updateValue(rawText);
			}
		};
		reader.readAsText(file);
		input.value = '';
	}

	// Generate a unique ID for the accordion item
	const id = crypto.randomUUID();
</script>

<Accordion>
	<Accordion.Item value={id}>
		<Accordion.ItemTrigger class="flex w-full items-center justify-between py-2 text-left">
			<div class="flex items-center gap-2">
				<FileText size={20} />
				<span class="font-bold">{label}</span>
			</div>
			
			<div class="flex items-center gap-2">
				<label class="variant-soft-primary btn btn-sm cursor-pointer" onclick={(e) => e.stopPropagation()}>
					<Upload size={14} class="mr-2" />
					Load File
					<input type="file" class="hidden" accept=".pem,.crt,.key" onchange={handleFileSelect} />
				</label>
				<Accordion.ItemIndicator>
					<ChevronDown class="h-4 w-4 transition-transform duration-200" />
				</Accordion.ItemIndicator>
			</div>
		</Accordion.ItemTrigger>
		<Accordion.ItemContent>
			<div class="p-2">
				<textarea
					class="textarea h-64 w-full font-mono text-xs"
					bind:value={rawText}
					oninput={handleInput}
					placeholder="Paste PEM content here (-----BEGIN...)"
				></textarea>
				{#if helperText}
					<div class="mt-2 text-xs opacity-70">{helperText}</div>
				{/if}
			</div>
		</Accordion.ItemContent>
	</Accordion.Item>
</Accordion>
