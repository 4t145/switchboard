import type { GetEntryResponse, GetRootsResponse } from '../types';
import { fetchQuery } from './index';

export const fileBrowserApi = {
	getRoots(): Promise<GetRootsResponse> {
		return fetchQuery<GetRootsResponse>('/api/file_browser/roots');
	},
	getEntry(params: {
		root: string;
		relativePath?: string;
		listChildren?: boolean;
		includeHidden?: boolean;
	}): Promise<GetEntryResponse> {
		const query = new URLSearchParams();
		query.append('root', params.root);
		if (params.relativePath && params.relativePath.length > 0) {
			query.append('relative_path', params.relativePath);
		}
		if (params.listChildren) {
			query.append('list_children', 'true');
		}
		if (params.includeHidden) {
			query.append('include_hidden', 'true');
		}
		return fetchQuery<GetEntryResponse>('/api/file_browser/entry', query);
	}
};
