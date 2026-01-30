<script lang="ts">
	import type { Snippet } from 'svelte';
	import type { FileStyleTls } from '$lib/api/types/human_readable';
	import { Accordion } from '@skeletonlabs/skeleton-svelte';
	import { Lock, Shield, ChevronDown, Calendar, User, Key } from '@lucide/svelte';
	import LinkOrValueDisplay from './link-or-value-display.svelte';
	import TlsOptionsDisplay from './tls-options-display.svelte';
	import { X509Certificate } from '@peculiar/x509';

	interface Props {
		tls: FileStyleTls;
	}

	let { tls }: Props = $props();

	// Check if this is SNI (multi-domain) TLS config
	const isSni = $derived('sni' in tls && Array.isArray(tls.sni));
	const sniDomains = $derived(isSni && 'sni' in tls ? tls.sni : []);

	// For single certificate
	const hasCerts = $derived('certs' in tls);
	const hasKey = $derived('key' in tls);

	let accordionOpen = $state<string[]>([]);

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

	// Helper function to detect PEM content
	function looksLikePem(content: string): boolean {
		return (
			content.includes('-----BEGIN CERTIFICATE-----') ||
			content.includes('-----BEGIN PRIVATE KEY-----') ||
			content.includes('-----BEGIN RSA PRIVATE KEY-----')
		);
	}

	// Helper function to get PEM type label
	function getPemTypeLabel(content: string): string {
		if (content.includes('-----BEGIN CERTIFICATE-----')) {
			return 'Certificate file detected';
		} else if (
			content.includes('-----BEGIN PRIVATE KEY-----') ||
			content.includes('-----BEGIN RSA PRIVATE KEY-----')
		) {
			return 'Private key file detected';
		}
		return 'Cryptographic file detected';
	}

	// Parse certificate from PEM
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
				console.warn('Failed to parse Subject Alternative Names:', e, pemContent);
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
	function getCertStatus(validTo: string): {
		status: 'valid' | 'expiring' | 'expired';
		message: string;
	} {
		const expiryDate = new Date(validTo);
		const now = new Date();
		const daysUntilExpiry = Math.floor(
			(expiryDate.getTime() - now.getTime()) / (1000 * 60 * 60 * 24)
		);

		if (daysUntilExpiry < 0) {
			return { status: 'expired', message: 'Expired' };
		} else if (daysUntilExpiry < 30) {
			return { status: 'expiring', message: `Expires in ${daysUntilExpiry} days` };
		} else {
			return { status: 'valid', message: 'Valid' };
		}
	}
</script>

{#snippet pemContentDisplay({ content }: { content: string })}
	{#if looksLikePem(content)}
		<!-- Check if it's a certificate or private key -->
		{@const isCertificate = content.includes('-----BEGIN CERTIFICATE-----')}
		{@const certInfo = isCertificate ? parseCertificate(content) : null}

		<!-- PEM file display -->
		<div class="rounded bg-surface-100 p-4 dark:bg-surface-800">
			<div class="mb-3 flex items-center gap-2 text-sm text-surface-700 dark:text-surface-300">
				{#if isCertificate}
					<Lock class="h-4 w-4 text-primary-500" />
				{:else}
					<Key class="h-4 w-4 text-warning-500" />
				{/if}
				<span class="font-medium">{getPemTypeLabel(content)}</span>
			</div>

			{#if certInfo}
				<!-- Certificate Information -->
				{@const certStatus = getCertStatus(certInfo.validTo)}

				<div class="mb-4 space-y-3">
					<!-- Status Badge -->
					<div class="flex items-center gap-2">
						<span
							class="rounded px-2 py-1 text-xs font-medium {certStatus.status === 'valid'
								? 'bg-success-100 text-success-700 dark:bg-success-900 dark:text-success-300'
								: certStatus.status === 'expiring'
									? 'bg-warning-100 text-warning-700 dark:bg-warning-900 dark:text-warning-300'
									: 'bg-error-100 text-error-700 dark:bg-error-900 dark:text-error-300'}"
						>
							{certStatus.message}
						</span>
					</div>

					<!-- Subject -->
					<div class="space-y-1 text-xs">
						<div class="flex items-center gap-1.5 text-surface-600 dark:text-surface-400">
							<User class="h-3.5 w-3.5" />
							<span class="font-medium">Subject:</span>
						</div>
						<div class="pl-5 font-mono break-all text-surface-700 dark:text-surface-300">
							{certInfo.subject}
						</div>
					</div>

					<!-- Issuer -->
					<div class="space-y-1 text-xs">
						<div class="flex items-center gap-1.5 text-surface-600 dark:text-surface-400">
							<Shield class="h-3.5 w-3.5" />
							<span class="font-medium">Issuer:</span>
						</div>
						<div class="pl-5 font-mono break-all text-surface-700 dark:text-surface-300">
							{certInfo.issuer}
						</div>
					</div>

					<!-- Validity Period -->
					<div class="space-y-1 text-xs">
						<div class="flex items-center gap-1.5 text-surface-600 dark:text-surface-400">
							<Calendar class="h-3.5 w-3.5" />
							<span class="font-medium">Valid Period:</span>
						</div>
						<div class="space-y-0.5 pl-5 text-surface-700 dark:text-surface-300">
							<div>From: {formatDate(certInfo.validFrom)}</div>
							<div>To: {formatDate(certInfo.validTo)}</div>
						</div>
					</div>

					<!-- Subject Alternative Names -->
					{#if certInfo.subjectAltNames && certInfo.subjectAltNames.length > 0}
						<div class="space-y-1 text-xs">
							<div class="font-medium text-surface-600 dark:text-surface-400">
								Subject Alternative Names:
							</div>
							<div class="space-y-0.5 pl-5 text-surface-700 dark:text-surface-300">
								{#each certInfo.subjectAltNames as san}
									<div class="font-mono">• {san}</div>
								{/each}
							</div>
						</div>
					{/if}

					<!-- Technical Details -->
					<details class="text-xs">
						<summary
							class="cursor-pointer text-primary-600 hover:text-primary-700 dark:text-primary-400 dark:hover:text-primary-300"
						>
							View technical details
						</summary>
						<div class="mt-2 space-y-2 pl-4 text-surface-700 dark:text-surface-300">
							<div>
								<span class="text-surface-600 dark:text-surface-400">Serial Number:</span>
								<span class="ml-2 font-mono">{certInfo.serialNumber}</span>
							</div>
							<div>
								<span class="text-surface-600 dark:text-surface-400">Signature Algorithm:</span>
								<span class="ml-2 font-mono">{certInfo.signatureAlgorithm}</span>
							</div>
							<div>
								<span class="text-surface-600 dark:text-surface-400">Public Key Algorithm:</span>
								<span class="ml-2 font-mono">{certInfo.publicKeyAlgorithm}</span>
							</div>
						</div>
					</details>
				</div>
			{:else if isCertificate}
				<!-- Certificate parsing failed -->
				<p class="mb-3 text-xs text-warning-600 dark:text-warning-400">
					Failed to parse certificate. The raw PEM data is available below.
				</p>
			{:else}
				<!-- Private key - don't parse -->
				<p class="mb-3 text-xs text-surface-600 dark:text-surface-400">
					Private key data (content hidden for security).
				</p>
			{/if}

			<!-- Collapsible raw PEM view -->
			<details class="text-xs">
				<summary
					class="cursor-pointer text-primary-600 hover:text-primary-700 dark:text-primary-400 dark:hover:text-primary-300"
				>
					View raw PEM data
				</summary>
				<div class="mt-2 overflow-x-auto rounded bg-surface-200 p-3 dark:bg-surface-900">
					<pre class="font-mono text-xs"><code>{content}</code></pre>
				</div>
			</details>
		</div>
	{:else}
		<!-- Normal content -->
		<div class="overflow-x-auto rounded bg-surface-100 p-4 dark:bg-surface-800">
			<pre class="font-mono text-xs"><code>{content}</code></pre>
		</div>
	{/if}
{/snippet}

<div class="space-y-3 card p-4">
	<!-- TLS header -->
	<div class="flex items-center gap-2">
		<Lock class="h-5 w-5 flex-shrink-0 text-primary-500" />
		<h3 class="text-lg font-semibold">{tls.name}</h3>
	</div>

	<!-- Type badge -->
	<div class="flex items-center gap-2">
		<Shield class="h-4 w-4 text-surface-500" />
		<span class="text-sm font-medium text-surface-700 dark:text-surface-300">
			{#if isSni}
				SNI Multi-Certificate ({sniDomains.length} domains)
			{:else}
				Single Certificate
			{/if}
		</span>
	</div>

	{#if isSni}
		<!-- SNI domains list -->
		<Accordion
			collapsible
			value={accordionOpen}
			onValueChange={(details) => (accordionOpen = details.value)}
		>
			<Accordion.Item value="domains">
				<Accordion.ItemTrigger
					class="preset-ghost-surface btn w-full justify-between gap-2 btn-sm text-left"
				>
					{accordionOpen.includes('domains') ? 'Hide domains' : 'View all domains'}
					<Accordion.ItemIndicator class="group">
						<ChevronDown class="h-4 w-4 transition group-data-[state=open]:rotate-180" />
					</Accordion.ItemIndicator>
				</Accordion.ItemTrigger>
				<Accordion.ItemContent class="mt-2 space-y-3">
					{#each sniDomains as domain}
						<div class="space-y-2 border-l-2 border-primary-300 pl-4 dark:border-primary-700">
							<div class="text-sm font-medium">{domain.hostname}</div>
							{#if domain.certs}
								<div class="text-sm">
									<span class="text-surface-600 dark:text-surface-400">Certs:</span>
									<div class="mt-1">
										<LinkOrValueDisplay
											value={domain.certs}
											resolveContent="string"
											dataType="PemsFile"
										/>
									</div>
								</div>
							{/if}
							{#if domain.key}
								<div class="text-sm">
									<span class="text-surface-600 dark:text-surface-400">Key:</span>
									<div class="mt-1">
										<LinkOrValueDisplay
											value={domain.key}
											resolveContent="string"
											dataType="PemFile"
										/>
									</div>
								</div>
							{/if}
						</div>
					{/each}
				</Accordion.ItemContent>
			</Accordion.Item>
		</Accordion>
	{:else}
		<!-- Single certificate -->
		<div class="space-y-2">
			{#if hasCerts && 'certs' in tls}
				<div class="text-sm">
					<span class="font-medium text-surface-600 dark:text-surface-400">Certificate:</span>
					<div class="mt-1 pl-2">
						<LinkOrValueDisplay value={tls.certs} resolveContent="string" dataType="PemsFile" />
					</div>
				</div>
			{/if}
			{#if hasKey && 'key' in tls}
				<div class="text-sm">
					<span class="font-medium text-surface-600 dark:text-surface-400">Private Key:</span>
					<div class="mt-1 pl-2">
						<LinkOrValueDisplay value={tls.key} resolveContent="string" dataType="PemFile" />
					</div>
				</div>
			{/if}
		</div>
	{/if}

	<!-- TLS Options (if present) -->
	{#if tls.options}
		<div class="border-t border-surface-200 pt-2 dark:border-surface-700">
			<TlsOptionsDisplay options={tls.options} />
		</div>
	{/if}
</div>
