<script lang="ts">
	import { Collapsible ,FileUpload } from '@skeletonlabs/skeleton-svelte';
	import { Upload, FileText, ChevronDown, ChevronRight, CheckCircle, AlertTriangle, XCircle, FileIcon } from 'lucide-svelte';
	import { slide } from 'svelte/transition';

	type ValidationRule = {
		validate: (value: string) => boolean;
		message: string;
	};

	type ValidationResult = {
		isValid: boolean;
		errors: string[];
		warnings: string[];
	};

	let {
		value = $bindable(),
		label = 'PEM Certificate',
		helperText = '',
		required = false,
		validationRules = []
	} = $props<{
		value: string ;
		label?: string;
		helperText?: string;
		required?: boolean;
		validationRules?: ValidationRule[];
	}>();

	// Internal text state
	let rawText = $state(value);
	let open = $state(false);
	let validationResult = $state<ValidationResult>({ isValid: true, errors: [], warnings: [] });

	// PEM validation regex patterns
	const PEM_HEADER_REGEX = /-----BEGIN\s+([A-Z\s]+)-----/;
	const PEM_FOOTER_REGEX = /-----END\s+([A-Z\s]+)-----/;
	const PEM_FULL_REGEX = /-----BEGIN\s+([A-Z\s]+)-----[\s\S]*?-----END\s+([A-Z\s]+)-----/g;
	const BASE64_REGEX = /^[A-Za-z0-9+/\s]*={0,2}$/;

	// Built-in validation rules
	const builtInRules: ValidationRule[] = [
		{
			validate: (text: string) => {
				if (!required) return true;
				return text.trim().length > 0;
			},
			message: 'This field is required'
		},
		{
			validate: (text: string) => {
				if (!text.trim()) return true; // Empty is OK if not required
				const blocks = extractPems(text);
				return blocks.length > 0;
			},
			message: 'Must contain at least one valid PEM block'
		},
		{
			validate: (text: string) => {
				if (!text.trim()) return true;
				const blocks = extractPems(text);
				return blocks.every(block => validatePemBlock(block));
			},
			message: 'All PEM blocks must have matching BEGIN/END headers'
		}
	];

	// Combined validation rules
	const allRules = $derived([...builtInRules, ...validationRules]);

	// Update validation when text changes
	$effect(() => {
		validationResult = validateInput(rawText);
	});

	// Update value when text changes
	function handleInput(e: Event) {
		const target = e.target as HTMLTextAreaElement;
		rawText = target.value;
		updateValue(rawText);
	}

	function updateValue(text: string) {
		value = text;
	}

	function validateInput(text: string): ValidationResult {
		const errors: string[] = [];
		const warnings: string[] = [];

		// Run all validation rules
		for (const rule of allRules) {
			if (!rule.validate(text)) {
				errors.push(rule.message);
			}
		}

		// Additional warnings
		if (text.trim() && !text.includes('-----BEGIN')) {
			warnings.push('Content does not appear to be in PEM format');
		}

		const isValid = errors.length === 0;
		return { isValid, errors, warnings };
	}

	function validatePemBlock(block: string): boolean {
		const lines = block.split('\n').map(line => line.trim()).filter(line => line);
		
		if (lines.length < 3) return false; // Must have at least BEGIN, content, END
		
		const beginMatch = lines[0].match(PEM_HEADER_REGEX);
		const endMatch = lines[lines.length - 1].match(PEM_FOOTER_REGEX);
		
		if (!beginMatch || !endMatch) return false;
		
		// Check that BEGIN and END headers match
		const beginType = beginMatch[1].trim();
		const endType = endMatch[1].trim();
		if (beginType !== endType) return false;
		
		// Check that content lines are valid base64
		const contentLines = lines.slice(1, -1);
		const content = contentLines.join('');
		return BASE64_REGEX.test(content);
	}

	function extractPems(text: string): string[] {
		const matches = text.match(PEM_FULL_REGEX);
		return matches || [];
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

<Collapsible open={open} onOpenChange={(details) => (open = details.open)}>
	<Collapsible.Trigger class="flex w-full items-center justify-between py-2 text-left">
		<div class="flex items-center gap-2">
			<FileText size={20} />
			<span class="font-bold">{label}</span>
			{#if required}
				<span class="text-error-500">*</span>
			{/if}
		</div>
		<div class="flex items-center gap-2">
			<!-- Validation Status Icon -->
			{#if rawText.trim()}
				{#if validationResult.isValid}
					<CheckCircle size={16} class="text-success-500" />
				{:else if validationResult.errors.length > 0}
					<XCircle size={16} class="text-error-500" />
				{:else}
					<AlertTriangle size={16} class="text-warning-500" />
				{/if}
			{/if}
			
			{#if open}
				<ChevronDown class="h-4 w-4 transition-transform duration-200" />
			{:else}
				<ChevronDown class="h-4 w-4 rotate-90 transition-transform duration-200" />
			{/if}
		</div>

	</Collapsible.Trigger>
	<Collapsible.Content class="flex flex-col w-full justify-between py-2 text-left">

		<!-- Textarea with validation styling -->
		<textarea
			class="textarea h-64 w-full font-mono text-xs {validationResult.isValid ? '' : 'border-error-500 focus:border-error-500'}"
			bind:value={rawText}
			oninput={handleInput}
			placeholder="Paste PEM content here (-----BEGIN...)"
			aria-invalid={!validationResult.isValid}
			aria-describedby={validationResult.errors.length > 0 ? `${id}-errors` : undefined}
		></textarea>

		<!-- Validation Messages -->
		{#if validationResult.errors.length > 0}
			<div id="{id}-errors" class="mt-2">
				{#each validationResult.errors as error}
					<div class="flex items-center gap-2 text-sm text-error-600 dark:text-error-400">
						<XCircle size={14} />
						<span>{error}</span>
					</div>
				{/each}
			</div>
		{/if}

		{#if validationResult.warnings.length > 0}
			<div class="mt-2">
				{#each validationResult.warnings as warning}
					<div class="flex items-center gap-2 text-sm text-warning-600 dark:text-warning-400">
						<AlertTriangle size={14} />
						<span>{warning}</span>
					</div>
				{/each}
			</div>
		{/if}

		<!-- Helper Text -->
		{#if helperText !== ''}
			<p class="mt-2 text-sm text-surface-600 dark:text-surface-400">{helperText}</p>
		{/if}

		<!-- PEM Block Info -->
		{#if rawText.trim() && validationResult.isValid}
			<div class="mt-2 text-sm text-success-600 dark:text-success-400">
				<div class="flex items-center gap-2">
					<CheckCircle size={14} />
					<span>{extractPems(rawText).length} valid PEM block(s) detected</span>
				</div>
			</div>
		{/if}
	</Collapsible.Content>
</Collapsible>