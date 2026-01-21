<script lang="ts">
	import { Collapsible ,FileUpload } from '@skeletonlabs/skeleton-svelte';
	import { Upload, FileText, ChevronDown, ChevronRight, CheckCircle, AlertTriangle, XCircle, FileIcon, Lock, Shield, Calendar, User, Key } from 'lucide-svelte';
	import { slide } from 'svelte/transition';
	import { X509Certificate } from '@peculiar/x509';

	type ValidationRule = {
		validate: (value: string) => boolean;
		message: string;
	};

	type ValidationResult = {
		isValid: boolean;
		errors: string[];
		warnings: string[];
	};

	// Certificate info interface
	interface CertInfo {
		subject: string;
		issuer: string;
		validFrom: string;
		validTo: string;
		serialNumber: string;
		signatureAlgorithm: string;
		publicKeyAlgorithm: string;
		subjectAltNames?: string[];
	}

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

	// Internal text state - ensure value is always a string
	let rawText = $state(typeof value === 'string' ? value : '');
	let open = $state(false);
	let validationResult = $state<ValidationResult>({ isValid: true, errors: [], warnings: [] });

	// Certificate parsing state
	let certInfo = $state<CertInfo | null>(null);
	let isCertificate = $derived(rawText.includes('-----BEGIN CERTIFICATE-----'));
	let isPrivateKey = $derived(
		rawText.includes('-----BEGIN PRIVATE KEY-----') ||
		rawText.includes('-----BEGIN RSA PRIVATE KEY-----') ||
		rawText.includes('-----BEGIN EC PRIVATE KEY-----')
	);

	// PEM validation regex patterns
	const PEM_HEADER_REGEX = /-----BEGIN\s+([A-Z\s]+)-----/;
	const PEM_FOOTER_REGEX = /-----END\s+([A-Z\s]+)-----/;
	const PEM_FULL_REGEX = /-----BEGIN\s+([A-Z\s]+)-----[\s\S]*?-----END\s+([A-Z\s]+)-----/g;
	const BASE64_REGEX = /^[A-Za-z0-9+/\s]*={0,2}$/;

	// Certificate parsing functions
	function parseCertificate(pemContent: string): CertInfo | null {
		try {
			// Extract the first certificate from the PEM content
			const certMatch = pemContent.match(
				/-----BEGIN CERTIFICATE-----[\s\S]+?-----END CERTIFICATE-----/
			);
			if (!certMatch) return null;

			const cert = new X509Certificate(certMatch[0]);

			// Extract subject alternative names
			let subjectAltNames: string[] | undefined;
			try {
				const sanExt = cert.getExtension('2.5.29.17'); // subjectAltName OID
				if (sanExt) {
					// Parse SAN extension (simplified - may need refinement)
					const sanValue = sanExt.value;
					if (sanValue && typeof sanValue === 'object') {
						subjectAltNames = [];
						// @ts-ignore - SAN parsing is complex, this is a basic implementation
						for (const name of sanValue) {
							if (name.dNSName) subjectAltNames.push(name.dNSName);
						}
					}
				}
			} catch (e) {
				console.warn('Failed to parse Subject Alternative Names:', e);
				// SAN parsing failed, ignore
			}

			return {
				subject: cert.subject,
				issuer: cert.issuer,
				validFrom: cert.notBefore.toISOString(),
				validTo: cert.notAfter.toISOString(),
				serialNumber: cert.serialNumber,
				signatureAlgorithm: cert.signatureAlgorithm.name,
				publicKeyAlgorithm: cert.publicKey.algorithm.name,
				subjectAltNames
			};
		} catch (error) {
			console.error('Failed to parse certificate:', error);
			return null;
		}
	}

	// Format date for display
	function formatDate(isoDate: string): string {
		try {
			const date = new Date(isoDate);
			return date.toLocaleDateString('en-US', {
				year: 'numeric',
				month: 'short',
				day: 'numeric',
				hour: '2-digit',
				minute: '2-digit'
			});
		} catch {
			return isoDate;
		}
	}

	// Check if certificate is expired or expiring soon
	function getCertStatus(validTo: string): { status: 'valid' | 'expiring' | 'expired'; message: string } {
		const expiryDate = new Date(validTo);
		const now = new Date();
		const daysUntilExpiry = Math.floor((expiryDate.getTime() - now.getTime()) / (1000 * 60 * 60 * 24));

		if (daysUntilExpiry < 0) {
			return { status: 'expired', message: 'Expired' };
		} else if (daysUntilExpiry < 30) {
			return { status: 'expiring', message: `Expires in ${daysUntilExpiry} days` };
		} else {
			return { status: 'valid', message: 'Valid' };
		}
	}

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

	// Sync rawText when value changes externally
	$effect(() => {
		const stringValue = typeof value === 'string' ? value : '';
		if (stringValue !== rawText) {
			rawText = stringValue;
		}
	});

	// Update validation when text changes
	$effect(() => {
		validationResult = validateInput(rawText);
	});

	// Parse certificate when rawText contains valid certificate
	$effect(() => {
		if (isCertificate && rawText.trim()) {
			certInfo = parseCertificate(rawText);
		} else {
			certInfo = null;
		}
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
				{#if certInfo}
					{@const status = getCertStatus(certInfo.validTo)}
					{#if status.status === 'expired'}
						<XCircle size={16} class="text-error-500" />
					{:else if status.status === 'expiring'}
						<AlertTriangle size={16} class="text-warning-500" />
					{:else if validationResult.isValid}
						<CheckCircle size={16} class="text-success-500" />
					{/if}
				{:else if validationResult.isValid}
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

		<!-- Certificate Information Display -->
		{#if certInfo}
			{@const certStatus = getCertStatus(certInfo.validTo)}
			<div class="mt-4 p-4 bg-surface-100 dark:bg-surface-800 rounded-lg border border-surface-300 dark:border-surface-600 space-y-3">
				<!-- Status Badge -->
				<div class="flex items-center gap-2">
					<Lock size={16} class="text-primary-500" />
					<span class="font-medium text-sm">Certificate Information</span>
					<span
						class="text-xs px-2 py-1 rounded font-medium ml-auto {certStatus.status === 'valid'
							? 'bg-success-100 dark:bg-success-900 text-success-700 dark:text-success-300'
							: certStatus.status === 'expiring'
								? 'bg-warning-100 dark:bg-warning-900 text-warning-700 dark:text-warning-300'
								: 'bg-error-100 dark:bg-error-900 text-error-700 dark:text-error-300'}"
					>
						{certStatus.message}
					</span>
				</div>

				<!-- Subject -->
				<div class="text-xs space-y-1">
					<div class="flex items-center gap-1.5 text-surface-600 dark:text-surface-400">
						<User size={14} />
						<span class="font-medium">Subject:</span>
					</div>
					<div class="pl-5 text-surface-700 dark:text-surface-300 font-mono text-xs break-all">
						{certInfo.subject}
					</div>
				</div>

				<!-- Issuer -->
				<div class="text-xs space-y-1">
					<div class="flex items-center gap-1.5 text-surface-600 dark:text-surface-400">
						<Shield size={14} />
						<span class="font-medium">Issuer:</span>
					</div>
					<div class="pl-5 text-surface-700 dark:text-surface-300 font-mono text-xs break-all">
						{certInfo.issuer}
					</div>
				</div>

				<!-- Validity Period -->
				<div class="text-xs space-y-1">
					<div class="flex items-center gap-1.5 text-surface-600 dark:text-surface-400">
						<Calendar size={14} />
						<span class="font-medium">Valid Period:</span>
					</div>
					<div class="pl-5 text-surface-700 dark:text-surface-300 space-y-0.5">
						<div>From: {formatDate(certInfo.validFrom)}</div>
						<div>To: {formatDate(certInfo.validTo)}</div>
					</div>
				</div>

				<!-- Subject Alternative Names -->
				{#if certInfo.subjectAltNames && certInfo.subjectAltNames.length > 0}
					<div class="text-xs space-y-1">
						<div class="text-surface-600 dark:text-surface-400 font-medium">
							Subject Alternative Names:
						</div>
						<div class="pl-5 text-surface-700 dark:text-surface-300 space-y-0.5">
							{#each certInfo.subjectAltNames as san}
								<div class="font-mono text-xs">• {san}</div>
							{/each}
						</div>
					</div>
				{/if}

				<!-- Technical Details (Collapsible) -->
				<details class="text-xs">
					<summary
						class="cursor-pointer text-primary-600 dark:text-primary-400 hover:text-primary-700 dark:hover:text-primary-300 font-medium"
					>
						View technical details
					</summary>
					<div class="mt-2 pl-4 space-y-2 text-surface-700 dark:text-surface-300">
						<div>
							<span class="text-surface-600 dark:text-surface-400">Serial Number:</span>
							<span class="font-mono ml-2 text-xs">{certInfo.serialNumber}</span>
						</div>
						<div>
							<span class="text-surface-600 dark:text-surface-400">Signature Algorithm:</span>
							<span class="font-mono ml-2 text-xs">{certInfo.signatureAlgorithm}</span>
						</div>
						<div>
							<span class="text-surface-600 dark:text-surface-400">Public Key Algorithm:</span>
							<span class="font-mono ml-2 text-xs">{certInfo.publicKeyAlgorithm}</span>
						</div>
					</div>
				</details>
			</div>
		{:else if isCertificate && rawText.trim() && validationResult.isValid}
			<!-- Certificate parsing failed -->
			<div class="mt-4 p-3 bg-warning-100 dark:bg-warning-900 rounded-lg border border-warning-300 dark:border-warning-600">
				<div class="flex items-center gap-2 text-warning-700 dark:text-warning-300 text-sm">
					<AlertTriangle size={16} />
					<span>Failed to parse certificate details. The PEM format is valid but certificate information cannot be extracted.</span>
				</div>
			</div>
		{:else if isPrivateKey && rawText.trim()}
			<!-- Private Key Info (No Parsing) -->
			<div class="mt-4 p-3 bg-surface-100 dark:bg-surface-800 rounded-lg border border-surface-300 dark:border-surface-600">
				<div class="flex items-center gap-2 text-surface-700 dark:text-surface-300 text-sm">
					<Key size={16} class="text-warning-500" />
					<span>Private key detected (content details hidden for security)</span>
				</div>
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
					<span>
						{extractPems(rawText).length} valid PEM block(s) detected
						{#if isCertificate}
							(Certificate)
						{:else if isPrivateKey}
							(Private Key)
						{/if}
					</span>
				</div>
			</div>
		{/if}
	</Collapsible.Content>
</Collapsible>