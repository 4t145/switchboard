export type Indexed<T> = {
	id: string;
	data: T;
};

export type Cursor = {
	next: string | null;
};

export type PagedResult<T> = {
	items: Indexed<T>[];
	next_cursor: Cursor | null;
};

export type PageQuery = {
	limit: number;
	cursor: Cursor;
};

export type FlattenPageQueryWithFilter<T> = {
	limit: number;
	next: string | null;
} & T;
