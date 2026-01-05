import { fetchJson, fetchQuery } from '.';
import type { StorageObjectDescriptor, StorageObjectWithoutData } from '../types';
import type { PagedResult, PageQuery } from '../types/paged';

export type SaveStorageObjectRequest = {
	resolver: string;
	save_as?: string;
	config: unknown;
};

export type BatchDeleteStorageObjectRequest =
	| {
			mode: 'by_descriptor';
			objects: StorageObjectDescriptor[];
	  }
	| {
			mode: 'by_id';
			id: string;
	  };

export type ObjectFilter = {
	data_type?: string;
	id?: string;
	revision?: string;
	latest_only?: boolean;
	created_before?: Date;
	created_after?: Date;
};

export const storageApi = {
	get: (descriptor: StorageObjectDescriptor) =>
		fetchQuery<unknown>('/api/storage/object', new URLSearchParams(descriptor)),
	save: (request: SaveStorageObjectRequest) =>
		fetchJson<StorageObjectDescriptor>('/api/storage/object', {
			method: 'POST',
			body: JSON.stringify(request)
		}),
	delete: (descriptor: StorageObjectDescriptor) =>
		fetchQuery<void>('/api/storage/object', new URLSearchParams(descriptor), {
			method: 'DELETE'
		}),
	list(page: PageQuery, filter: ObjectFilter) {
		const params = new URLSearchParams({
			limit: page.limit.toString(),
			...(page.cursor.next ? { next: page.cursor.next } : {}),
			...(filter.data_type ? { data_type: filter.data_type } : {}),
			...(filter.id ? { id: filter.id } : {}),
			...(filter.revision ? { revision: filter.revision } : {}),
			...(filter.latest_only !== undefined ? { latest_only: filter.latest_only.toString() } : {}),
			...(filter.created_before ? { created_before: filter.created_before.toISOString() } : {}),
			...(filter.created_after ? { created_after: filter.created_after.toISOString() } : {})
		});
		return fetchQuery<PagedResult<StorageObjectWithoutData>>('/api/storage/objects', params);
	},
	batchDelete: (request: BatchDeleteStorageObjectRequest) =>
		fetchJson<void>('/api/storage/objects', {
			method: 'POST',
			body: JSON.stringify(request)
		})
};
