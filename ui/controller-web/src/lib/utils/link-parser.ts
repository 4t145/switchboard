export type LinkKind = 'storage' | 'file' | 'http';

export type ParsedLink = {
	kind: 'storage';
	scheme: 'storage';
	location: string;
} | {
	kind: 'file';
	scheme: 'file';
	location: string;
} | {
	kind: 'http';
	scheme: 'http' | 'https';
	location: string;
}

export type StorageObjectDescriptor = {
	id: string;
	revision: string;
};

const URI_SCHEMES = [
	'file://',
	'http://',
	'https://',
	'ftp://',
	'ftps://',
	'storage://'
];

export function isValidURI(str: string): boolean {
	for (const scheme of URI_SCHEMES) {
		if (str.startsWith(scheme)) {
			return true;
		}
	}

	return false;
}

export function parseLink(val: unknown): ParsedLink | null {
	if (typeof val !== 'string') {
		return null;
	}

	if (!isValidURI(val)) {
		return null;
	}

	if (val.startsWith('file://')) {
		return { kind: 'file', location: val.slice(7), scheme: 'file' };
	}

	if (val.startsWith('http://') || val.startsWith('https://')) {
		return { kind: 'http', location: val, scheme: val.startsWith('http://') ? 'http' : 'https' };
	}

	if (val.startsWith('storage://')) {
		const parts = val.slice(10).split('#');
		if (parts.length >= 2) {
			return {
				kind: 'storage',
				scheme: 'storage',
				location: val.slice(10)
			};
		}
	}

	return null;
}

export function formatLink(parsedLink: ParsedLink): string {
	if (parsedLink.kind === 'file') {
		const path = parsedLink.location.toString();
		return path.startsWith('file://') ? path : `file://${path}`;
	}

	if (parsedLink.kind === 'http') {
		return parsedLink.location.toString();
	}

	if (parsedLink.kind === 'storage') {
		return `storage://${parsedLink.location}`;
	}

	return '';
}

export function isLinkValue(value: unknown): value is string {
	return parseLink(value) !== null;
}

export function isInlineValue<T>(value: T | string): value is T {
	return !isLinkValue(value);
}


export function getLinkKind(value: unknown): LinkKind | null {
	const parsed = parseLink(value);
	return parsed ? parsed.kind : null;
}

export function getScheme(value: string) {
	return value.split('://', 1)[0];
}