export type ShortcutAction =
	| 'delete'
	| 'save'
	| 'addNode'
	| 'addFilter'
	| 'focusSearch'
	| 'close'
	| 'confirm'
	| 'undo'
	| 'redo'
	| 'help'
	| 'none';

export type KeyboardShortcut = {
	key: string;
	ctrlKey?: boolean;
	shiftKey?: boolean;
	altKey?: boolean;
	metaKey?: boolean;
	action: ShortcutAction;
	description: string;
	context?: 'all' | 'editor' | 'config' | 'tree' | 'canvas';
};

export const DEFAULT_SHORTCUTS: KeyboardShortcut[] = [
	{
		key: 'Delete',
		action: 'delete',
		description: 'Delete selected item',
		context: 'all'
	},
	{
		key: 'Backspace',
		action: 'delete',
		description: 'Delete selected item',
		context: 'all'
	},
	{
		key: 's',
		ctrlKey: true,
		action: 'save',
		description: 'Save changes',
		context: 'all'
	},
	{
		key: 'z',
		ctrlKey: true,
		action: 'undo',
		description: 'Undo',
		context: 'all'
	},
	{
		key: 'z',
		ctrlKey: true,
		shiftKey: true,
		action: 'redo',
		description: 'Redo',
		context: 'all'
	},
	{
		key: 'y',
		ctrlKey: true,
		action: 'redo',
		description: 'Redo',
		context: 'all'
	},
	{
		key: 'n',
		ctrlKey: true,
		action: 'addNode',
		description: 'Add new node',
		context: 'editor'
	},
	{
		key: 'f',
		ctrlKey: true,
		action: 'focusSearch',
		description: 'Focus search',
		context: 'tree'
	},
	{
		key: 'Escape',
		action: 'close',
		description: 'Close panel/dialog',
		context: 'all'
	},
	{
		key: 'Enter',
		action: 'confirm',
		description: 'Confirm action',
		context: 'all'
	},
	{
		key: '/',
		action: 'help',
		description: 'Show shortcuts',
		context: 'all'
	}
];

export class KeyboardShortcutManager {
	private shortcuts: KeyboardShortcut[] = [];
	private handlers: Map<ShortcutAction, () => void> = new Map();
	private context: 'all' | 'editor' | 'config' | 'tree' | 'canvas' = 'all';
	private enabled = true;
	private keydownListener: ((e: KeyboardEvent) => void) | null = null;

	constructor(shortcuts: KeyboardShortcut[] = DEFAULT_SHORTCUTS) {
		this.shortcuts = shortcuts;
		this.bindGlobal();
	}

	setContext(context: 'all' | 'editor' | 'config' | 'tree' | 'canvas'): void {
		this.context = context;
	}

	setEnabled(enabled: boolean): void {
		this.enabled = enabled;
	}

	registerHandler(action: ShortcutAction, handler: () => void): void {
		this.handlers.set(action, handler);
	}

	unregisterHandler(action: ShortcutAction): void {
		this.handlers.delete(action);
	}

	private bindGlobal(): void {
		if (typeof window === 'undefined') return;

		this.keydownListener = (e: KeyboardEvent) => this.handleKeyDown(e);
		window.addEventListener('keydown', this.keydownListener);
	}

	private handleKeyDown(e: KeyboardEvent): void {
		if (!this.enabled) return;

		const activeElement = document.activeElement;
		const isInputFocused =
			activeElement?.tagName === 'INPUT' ||
			activeElement?.tagName === 'TEXTAREA' ||
			activeElement?.getAttribute('contenteditable') === 'true';

		if (isInputFocused && ['Escape', 'Enter'].indexOf(e.key) === -1) {
			return;
		}

		const shortcut = this.findMatchingShortcut(e);
		if (shortcut && this.handlers.has(shortcut.action)) {
			e.preventDefault();
			const handler = this.handlers.get(shortcut.action);
			handler?.();
		}
	}

	private findMatchingShortcut(e: KeyboardEvent): KeyboardShortcut | undefined {
		return this.shortcuts.find(
			(s) =>
				s.key.toLowerCase() === e.key.toLowerCase() &&
				!!s.ctrlKey === e.ctrlKey &&
				!!s.shiftKey === e.shiftKey &&
				!!s.altKey === e.altKey &&
				!!s.metaKey === e.metaKey &&
				(s.context === 'all' || s.context === this.context)
		);
	}

	getShortcut(action: ShortcutAction): KeyboardShortcut | undefined {
		return this.shortcuts.find((s) => s.action === action);
	}

	getShortcutsForContext(
		context?: 'all' | 'editor' | 'config' | 'tree' | 'canvas'
	): KeyboardShortcut[] {
		return this.shortcuts.filter(
			(s) => !context || s.context === 'all' || s.context === context
		);
	}

	formatShortcut(shortcut: KeyboardShortcut): string {
		const parts: string[] = [];

		if (shortcut.ctrlKey) parts.push('Ctrl');
		if (shortcut.altKey) parts.push('Alt');
		if (shortcut.shiftKey) parts.push('Shift');
		if (shortcut.metaKey) parts.push('Meta');

		const key = this.formatKey(shortcut.key);
		parts.push(key);

		return parts.join('+');
	}

	private formatKey(key: string): string {
		if (key === ' ') return 'Space';
		if (key === 'ArrowUp') return '↑';
		if (key === 'ArrowDown') return '↓';
		if (key === 'ArrowLeft') return '←';
		if (key === 'ArrowRight') return '→';
		return key.charAt(0).toUpperCase() + key.slice(1).toLowerCase();
	}

	destroy(): void {
		if (this.keydownListener) {
			window.removeEventListener('keydown', this.keydownListener);
		}
		this.handlers.clear();
	}
}

export function createShortcutManager(
	shortcuts?: KeyboardShortcut[]
): KeyboardShortcutManager {
	return new KeyboardShortcutManager(shortcuts);
}
