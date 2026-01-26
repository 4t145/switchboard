export type HistoryAction = 'add' | 'update' | 'delete';

export type HistoryEntry = {
	action: HistoryAction;
	targetId: string;
	targetType: 'node' | 'filter' | 'flow';
	previousData: any;
	newData: any;
	timestamp: number;
	description: string;
};

export class UndoManager {
	private history: HistoryEntry[] = [];
	private redoStack: HistoryEntry[] = [];
	private maxHistorySize = 50;
	private deleteUndoTimers: Map<string, ReturnType<typeof setTimeout>> = new Map();
	private deleteUndoTimeout = 10000;

	canUndo(): boolean {
		return this.history.length > 0;
	}

	canRedo(): boolean {
		return this.redoStack.length > 0;
	}

	getHistory(): HistoryEntry[] {
		return [...this.history];
	}

	recordAction(entry: HistoryEntry): void {
		this.history.push(entry);
		this.redoStack = [];

		if (this.history.length > this.maxHistorySize) {
			this.history.shift();
		}

		if (entry.action === 'delete') {
			this.scheduleDeleteUndoTimeout(entry.targetId);
		}
	}

	private scheduleDeleteUndoTimeout(targetId: string): void {
		const existingTimer = this.deleteUndoTimers.get(targetId);
		if (existingTimer) {
			clearTimeout(existingTimer);
		}

		const timer = setTimeout(() => {
			this.deleteUndoTimers.delete(targetId);
		}, this.deleteUndoTimeout);

		this.deleteUndoTimers.set(targetId, timer);
	}

	canUndoDelete(targetId: string): boolean {
		const lastEntry = this.history[this.history.length - 1];
		return (
			lastEntry?.action === 'delete' &&
			lastEntry.targetId === targetId &&
			this.deleteUndoTimers.has(targetId)
		);
	}

	getUndoDeleteTimeRemaining(targetId: string): number {
		const timer = this.deleteUndoTimers.get(targetId);
		if (!timer) return 0;

		return 0;
	}

	undo(
		flowData: {
			entrypoint: string;
			nodes: Record<string, any>;
			filters: Record<string, any>;
		}
	): {
		flowData: {
			entrypoint: string;
			nodes: Record<string, any>;
			filters: Record<string, any>;
		};
		description: string;
	} {
		if (this.history.length === 0) {
			throw new Error('No history to undo');
		}

		const entry = this.history.pop()!;
		this.redoStack.push(entry);

		let description = entry.description;

		switch (entry.action) {
			case 'add': {
				if (entry.targetType === 'node') {
					delete flowData.nodes[entry.targetId];
				} else if (entry.targetType === 'filter') {
					delete flowData.filters[entry.targetId];
				}
				description = `Deleted ${entry.targetType} '${entry.targetId}'`;
				break;
			}
			case 'update': {
				if (entry.targetType === 'node') {
					flowData.nodes[entry.targetId] = entry.previousData;
				} else if (entry.targetType === 'filter') {
					flowData.filters[entry.targetId] = entry.previousData;
				} else if (entry.targetType === 'flow') {
					Object.assign(flowData, entry.previousData);
				}
				description = `Restored ${entry.targetType} '${entry.targetId}'`;
				break;
			}
			case 'delete': {
				if (entry.targetType === 'node') {
					flowData.nodes[entry.targetId] = entry.previousData;
				} else if (entry.targetType === 'filter') {
					flowData.filters[entry.targetId] = entry.previousData;
				}
				this.deleteUndoTimers.delete(entry.targetId);
				description = `Restored deleted ${entry.targetType} '${entry.targetId}'`;
				break;
			}
		}

		return { flowData, description };
	}

	redo(
		flowData: {
			entrypoint: string;
			nodes: Record<string, any>;
			filters: Record<string, any>;
		}
	): {
		flowData: {
			entrypoint: string;
			nodes: Record<string, any>;
			filters: Record<string, any>;
		};
		description: string;
	} {
		if (this.redoStack.length === 0) {
			throw new Error('No actions to redo');
		}

		const entry = this.redoStack.pop()!;
		this.history.push(entry);

		let description = entry.description;

		switch (entry.action) {
			case 'add': {
				if (entry.targetType === 'node') {
					flowData.nodes[entry.targetId] = entry.newData;
				} else if (entry.targetType === 'filter') {
					flowData.filters[entry.targetId] = entry.newData;
				}
				description = `Re-added ${entry.targetType} '${entry.targetId}'`;
				break;
			}
			case 'update': {
				if (entry.targetType === 'node') {
					flowData.nodes[entry.targetId] = entry.newData;
				} else if (entry.targetType === 'filter') {
					flowData.filters[entry.targetId] = entry.newData;
				} else if (entry.targetType === 'flow') {
					Object.assign(flowData, entry.newData);
				}
				description = `Re-applied changes to ${entry.targetType} '${entry.targetId}'`;
				break;
			}
			case 'delete': {
				if (entry.targetType === 'node') {
					delete flowData.nodes[entry.targetId];
				} else if (entry.targetType === 'filter') {
					delete flowData.filters[entry.targetId];
				}
				description = `Re-deleted ${entry.targetType} '${entry.targetId}'`;
				break;
			}
		}

		return { flowData, description };
	}

	clear(): void {
		this.history = [];
		this.redoStack = [];
		this.deleteUndoTimers.forEach((timer) => clearTimeout(timer));
		this.deleteUndoTimers.clear();
	}

	destroy(): void {
		this.clear();
	}
}

export function createHistoryEntry(
	action: HistoryAction,
	targetId: string,
	targetType: 'node' | 'filter' | 'flow',
	previousData: any,
	newData: any,
	description?: string
): HistoryEntry {
	return {
		action,
		targetId,
		targetType,
		previousData: JSON.parse(JSON.stringify(previousData)),
		newData: JSON.parse(JSON.stringify(newData)),
		timestamp: Date.now(),
		description:
			description ||
			`${action === 'add' ? 'Added' : action === 'delete' ? 'Deleted' : 'Updated'} ${targetType} '${targetId}'`
	};
}
