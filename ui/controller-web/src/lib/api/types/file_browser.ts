export type EntryType = 'file' | 'directory' | 'symlink' | 'other';

export type FsEntry = {
	name: string;
	path: string;
	entry_type: EntryType;
	size: number | null;
	modified_unix_ms: number | null;
	readonly: boolean | null;
	has_children: boolean | null;
};

export type GetEntryResponse = {
	entry: FsEntry;
	children: FsEntry[] | null;
};

export type GetRootsResponse = {
	roots: string[];
};
