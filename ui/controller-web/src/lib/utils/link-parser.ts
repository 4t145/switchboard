export type LinkKind = 'storage' | 'file' | 'http';

export type ParsedLink = {
	kind: LinkKind;
	data: any;
};

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

export function parseLink(val: any): ParsedLink | null {
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

export function formatLink(kind: LinkKind, data: any): string {
	if (kind === 'file') {
		const path = data.toString();
		return path.startsWith('file://') ? path : `file://${path}`;
	}

	if (kind === 'http') {
		return data.toString();
	}

	if (kind === 'storage') {
		return `storage://${data.id}#${data.revision}`;
	}

	return '';
}

export function isLinkValue(value: any): boolean {
	return parseLink(value) !== null;
}

export function getLinkKind(value: any): LinkKind | null {
	const parsed = parseLink(value);
	return parsed ? parsed.kind : null;
}
