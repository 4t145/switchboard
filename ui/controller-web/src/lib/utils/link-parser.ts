export type LinkKind = 'storage' | 'file' | 'http';

export type ParsedLink = {
	kind: 'storage';
	data: {
		id: string;
		revision: string;
	}
} | {
	kind: 'file';
	data: string;
} | {
	kind: 'http';
	data: string;
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
		return { kind: 'file', data: val.slice(7) };
	}

	if (val.startsWith('http://') || val.startsWith('https://')) {
		return { kind: 'http', data: val };
	}

	if (val.startsWith('storage://')) {
		const parts = val.slice(10).split('#');
		if (parts.length >= 2) {
			return {
				kind: 'storage',
				data: { id: parts[0], revision: parts[1] } as StorageObjectDescriptor
			};
		}
	}

	if (val.startsWith('ftp://') || val.startsWith('ftps://')) {
		return { kind: 'http', data: val };
	}

	return null;
}

export function formatLink(parsedLink: ParsedLink): string {
	if (parsedLink.kind === 'file') {
		const path = parsedLink.data.toString();
		return path.startsWith('file://') ? path : `file://${path}`;
	}

	if (parsedLink.kind === 'http') {
		return parsedLink.data.toString();
	}

	if (parsedLink.kind === 'storage') {
		return `storage://${parsedLink.data.id}#${parsedLink.data.revision}`;
	}

	return '';
}

export function isLinkValue(value: unknown): value is string {
	return parseLink(value) !== null;
}

export function getLinkKind(value: unknown): LinkKind | null {
	const parsed = parseLink(value);
	return parsed ? parsed.kind : null;
}
