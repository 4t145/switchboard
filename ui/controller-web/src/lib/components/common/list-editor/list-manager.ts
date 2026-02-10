export type ListManager<T> = {
	swap(indexA: number, indexB: number): void;
	moveUp(index: number): void;
	moveDown(index: number): void;
	remove(index: number): void;
	insertAt(index: number, item: T): void;
	pushBack(item: T): void;
	pushFront(item: T): void;
	getValue(index: number): T;
	getList(): T[];
	getItem(index: number): ListManagerItem<T>;
	getItems(): ListManagerItem<T>[];
	length(): number;
}

export type ListManagerItem<T> = {
	index: number;
	value: T;
	rawIndex: number | undefined;
};
export const ListManager = {
    new<T>(items?: T[]): ListManager<T> {
        return useListManager(items ?? []);
    }
}

export function useListManager<T>(items: T[]): ListManager<T> {
	const list: ListManagerItem<T>[] = items.map((item, index) => ({
		index,
		value: item,
		rawIndex: index
	}));
	return {
		swap(indexA: number, indexB: number) {
			const temp = list[indexA];
			list[indexA] = list[indexB];
			list[indexB] = temp;
			list[indexA].index = indexA;
			list[indexB].index = indexB;
		},
		moveUp(index: number) {
			if (index > 0) {
				this.swap(index, index - 1);
			}
		},
		moveDown(index: number) {
			if (index < list.length - 1) {
				this.swap(index, index + 1);
			}
		},
		remove(index: number) {
			list.splice(index, 1);
			// Update indices
			for (let i = index; i < list.length; i++) {
				list[i].index = i;
			}
		},
		insertAt(index: number, item: T) {
			list.splice(index, 0, { index, value: item, rawIndex: undefined });
			// Update indices
			for (let i = index + 1; i < list.length; i++) {
				list[i].index = i;
			}
		},
		pushBack(item: T) {
			list.push({ index: list.length, value: item, rawIndex: undefined });
		},
		pushFront(item: T) {
			this.insertAt(0, item);
		},
		getValue(index: number): T {
			return list[index].value;
		},
		getList(): T[] {
			return list.map((item) => item.value);
		},
		getItem(index: number): ListManagerItem<T> {
			return list[index];
		},
		getItems(): ListManagerItem<T>[] {
			return list;
		},
		length(): number {
			return list.length;
		}
	};
}
