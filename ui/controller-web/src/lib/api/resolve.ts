/**
 * API client for link resolution
 */

/**
 * Resolve a link to its value
 * @param link - The link to resolve (e.g., "file://path/to/file")
 * @returns The resolved value as JSON
 */
export async function resolveLink(link: string): Promise<unknown> {
	const response = await fetch(`/api/resolve/value?link=${encodeURIComponent(link)}`);
	if (!response.ok) {
		throw new Error(`Failed to resolve link: ${response.statusText}`);
	}
	return await response.json();
}

/**
 * Resolve a link to its string content
 * @param link - The link to resolve
 * @returns The resolved content as a string
 */
export async function resolveLinkAsString(link: string): Promise<string> {
	const response = await fetch(`/api/resolve/string?link=${encodeURIComponent(link)}`);
	if (!response.ok) {
		throw new Error(`Failed to resolve link: ${response.statusText}`);
	}
	return await response.text();
}
