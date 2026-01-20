<script lang="ts">
	import type { Snippet } from 'svelte';
	import type { FileStyleTls } from '$lib/api/types/human_readable';
	import { Accordion } from '@skeletonlabs/skeleton-svelte';
	import { Lock, Shield, ChevronDown, Calendar, User, Key } from 'lucide-svelte';
	import LinkOrValueDisplay from './link-or-value-display.svelte';
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
</script>

{#snippet pemContentDisplay({ content }: { content: string })}
	{#if looksLikePem(content)}
		<!-- Check if it's a certificate or private key -->
		{@const isCertificate = content.includes('-----BEGIN CERTIFICATE-----')}
		{@const certInfo = isCertificate ? parseCertificate(content) : null}
		
		<!-- PEM file display -->
		<div class="bg-surface-100 dark:bg-surface-800 rounded p-4">
			<div class="flex items-center gap-2 text-sm text-surface-700 dark:text-surface-300 mb-3">
				{#if isCertificate}
					<Lock class="w-4 h-4 text-primary-500" />
				{:else}
					<Key class="w-4 h-4 text-warning-500" />
				{/if}
				<span class="font-medium">{getPemTypeLabel(content)}</span>
			</div>

			{#if certInfo}
				<!-- Certificate Information -->
				{@const certStatus = getCertStatus(certInfo.validTo)}
				
				<div class="space-y-3 mb-4">
					<!-- Status Badge -->
					<div class="flex items-center gap-2">
						<span
							class="text-xs px-2 py-1 rounded font-medium {certStatus.status === 'valid'
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
							<User class="w-3.5 h-3.5" />
							<span class="font-medium">Subject:</span>
						</div>
						<div class="pl-5 text-surface-700 dark:text-surface-300 font-mono break-all">
							{certInfo.subject}
						</div>
					</div>

					<!-- Issuer -->
					<div class="text-xs space-y-1">
						<div class="flex items-center gap-1.5 text-surface-600 dark:text-surface-400">
							<Shield class="w-3.5 h-3.5" />
							<span class="font-medium">Issuer:</span>
						</div>
						<div class="pl-5 text-surface-700 dark:text-surface-300 font-mono break-all">
							{certInfo.issuer}
						</div>
					</div>

					<!-- Validity Period -->
					<div class="text-xs space-y-1">
						<div class="flex items-center gap-1.5 text-surface-600 dark:text-surface-400">
							<Calendar class="w-3.5 h-3.5" />
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
									<div class="font-mono">• {san}</div>
								{/each}
							</div>
						</div>
					{/if}

					<!-- Technical Details -->
					<details class="text-xs">
						<summary
							class="cursor-pointer text-primary-600 dark:text-primary-400 hover:text-primary-700 dark:hover:text-primary-300"
						>
							View technical details
						</summary>
						<div class="mt-2 pl-4 space-y-2 text-surface-700 dark:text-surface-300">
							<div>
								<span class="text-surface-600 dark:text-surface-400">Serial Number:</span>
								<span class="font-mono ml-2">{certInfo.serialNumber}</span>
							</div>
							<div>
								<span class="text-surface-600 dark:text-surface-400">Signature Algorithm:</span>
								<span class="font-mono ml-2">{certInfo.signatureAlgorithm}</span>
							</div>
							<div>
								<span class="text-surface-600 dark:text-surface-400">Public Key Algorithm:</span>
								<span class="font-mono ml-2">{certInfo.publicKeyAlgorithm}</span>
							</div>
						</div>
					</details>
				</div>
			{:else if isCertificate}
				<!-- Certificate parsing failed -->
				<p class="text-xs text-warning-600 dark:text-warning-400 mb-3">
					Failed to parse certificate. The raw PEM data is available below.
				</p>
			{:else}
				<!-- Private key - don't parse -->
				<p class="text-xs text-surface-600 dark:text-surface-400 mb-3">
					Private key data (content hidden for security).
				</p>
			{/if}

			<!-- Collapsible raw PEM view -->
			<details class="text-xs">
				<summary
					class="cursor-pointer text-primary-600 dark:text-primary-400 hover:text-primary-700 dark:hover:text-primary-300"
				>
					View raw PEM data
				</summary>
				<div class="bg-surface-200 dark:bg-surface-900 rounded p-3 overflow-x-auto mt-2">
					<pre class="font-mono text-xs"><code>{content}</code></pre>
				</div>
			</details>
		</div>
	{:else}
		<!-- Normal content -->
		<div class="bg-surface-100 dark:bg-surface-800 rounded p-4 overflow-x-auto">
			<pre class="text-xs font-mono"><code>{content}</code></pre>
		</div>
	{/if}
{/snippet}

<div class="card p-4 space-y-3">
	<!-- TLS header -->
	<div class="flex items-center gap-2">
		<Lock class="w-5 h-5 text-primary-500 flex-shrink-0" />
		<h3 class="font-semibold text-lg">{tls.name}</h3>
	</div>

	<!-- Type badge -->
	<div class="flex items-center gap-2">
		<Shield class="w-4 h-4 text-surface-500" />
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
					class="btn btn-sm preset-ghost-surface w-full text-left justify-between gap-2"
				>
					{accordionOpen.includes('domains') ? 'Hide domains' : 'View all domains'}
					<Accordion.ItemIndicator class="group">
						<ChevronDown class="h-4 w-4 transition group-data-[state=open]:rotate-180" />
					</Accordion.ItemIndicator>
				</Accordion.ItemTrigger>
				<Accordion.ItemContent class="mt-2 space-y-3">
					{#each sniDomains as domain}
						<div class="pl-4 border-l-2 border-primary-300 dark:border-primary-700 space-y-2">
							<div class="font-medium text-sm">{domain.hostname}</div>
							{#if domain.certs}
								<div class="text-sm">
									<span class="text-surface-600 dark:text-surface-400">Certs:</span>
									<div class="mt-1">
										<LinkOrValueDisplay value={domain.certs} resolveContent={'string'}>
											{#snippet customDisplay({ content })}
												{@render pemContentDisplay({ content })}
											{/snippet}
										</LinkOrValueDisplay>
									</div>
								</div>
							{/if}
							{#if domain.key}
								<div class="text-sm">
									<span class="text-surface-600 dark:text-surface-400">Key:</span>
									<div class="mt-1">
										<LinkOrValueDisplay value={domain.key} resolveContent={'string'}>
											{#snippet customDisplay({ content })}
												{@render pemContentDisplay({ content })}
											{/snippet}
										</LinkOrValueDisplay>
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
					<span class="text-surface-600 dark:text-surface-400 font-medium">Certificate:</span>
					<div class="mt-1 pl-2">
						<LinkOrValueDisplay value={tls.certs} resolveContent={'string'}>
							{#snippet customDisplay({ content })}
								{@render pemContentDisplay({ content })}
							{/snippet}
						</LinkOrValueDisplay>
					</div>
				</div>
			{/if}
			{#if hasKey && 'key' in tls}
				<div class="text-sm">
					<span class="text-surface-600 dark:text-surface-400 font-medium">Private Key:</span>
					<div class="mt-1 pl-2">
						<LinkOrValueDisplay value={tls.key} resolveContent={'string'}>
							{#snippet customDisplay({ content })}
								{@render pemContentDisplay({ content })}
							{/snippet}
						</LinkOrValueDisplay>
					</div>
				</div>
			{/if}
		</div>
	{/if}

	<!-- TLS Options (if present) -->
	{#if tls.options}
		<div class="pt-2 border-t border-surface-300 dark:border-surface-700">
			<div class="text-sm font-medium text-surface-700 dark:text-surface-300 mb-1">
				TLS Options:
			</div>
			<code
				class="text-xs bg-surface-200 dark:bg-surface-700 px-2 py-1 rounded block overflow-x-auto"
			>
				{JSON.stringify(tls.options, null, 2)}
			</code>
		</div>
	{/if}
</div>
