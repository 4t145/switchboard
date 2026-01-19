/**
 * Custom Themes Store - Manages user-uploaded custom themes
 * Uses Svelte 5 runes for reactive state management
 */

import { parseThemeCSS, generateThemeId } from '$lib/utils/theme-parser';

const STORAGE_KEY = 'switchboard-custom-themes';
const MAX_THEMES = 10;

export interface CustomTheme {
	id: string;
	name: string;
	css: string;
	colors: string[];
	createdAt: number;
}

/**
 * Custom Themes Store Class
 */
class CustomThemesStore {
	private themes = $state<CustomTheme[]>([]);
	private injectedThemeId = $state<string | null>(null);
	private initialized = $state(false);

	constructor() {
		// Load themes on client side only
		if (typeof window !== 'undefined') {
			this.load();
			this.initialized = true;
		}
	}

	/**
	 * Load custom themes from localStorage
	 */
	private load(): void {
		if (typeof window === 'undefined') return;

		try {
			const stored = localStorage.getItem(STORAGE_KEY);
			if (stored) {
				const parsed = JSON.parse(stored);
				if (Array.isArray(parsed)) {
					this.themes = parsed;
				}
			}
		} catch (error) {
			console.error('Failed to load custom themes from localStorage:', error);
			this.themes = [];
		}
	}

	/**
	 * Save custom themes to localStorage
	 */
	private save(): void {
		if (typeof window === 'undefined') return;

		try {
			localStorage.setItem(STORAGE_KEY, JSON.stringify(this.themes));
		} catch (error) {
			console.error('Failed to save custom themes to localStorage:', error);
		}
	}

	/**
	 * Add a new custom theme from a file
	 */
	async addTheme(file: File): Promise<{ success: boolean; error?: string; theme?: CustomTheme }> {
		// Check theme limit
		if (this.themes.length >= MAX_THEMES) {
			return { success: false, error: 'Maximum theme limit reached' };
		}

		// Read file content
		try {
			const content = await file.text();

			// Parse and validate theme
			const parsed = parseThemeCSS(content, file.name);

			if (!parsed.isValid) {
				return { success: false, error: parsed.error || 'Invalid theme format' };
			}

			// Check for duplicate name
			if (this.themes.some((t) => t.name.toLowerCase() === parsed.name.toLowerCase())) {
				// Auto-rename with timestamp
				parsed.name = `${parsed.name} (${new Date().toLocaleDateString()})`;
			}

			// Create theme object
			const theme: CustomTheme = {
				id: generateThemeId(),
				name: parsed.name,
				css: parsed.css,
				colors: parsed.colors,
				createdAt: Date.now()
			};

			// Add to themes array
			this.themes = [...this.themes, theme];
			this.save();

			return { success: true, theme };
		} catch (error) {
			console.error('Error reading theme file:', error);
			return { success: false, error: 'Failed to read file' };
		}
	}

	/**
	 * Remove a custom theme
	 */
	removeTheme(id: string): boolean {
		const initialLength = this.themes.length;
		this.themes = this.themes.filter((t) => t.id !== id);

		if (this.themes.length < initialLength) {
			// If this theme was currently injected, remove it
			if (this.injectedThemeId === id) {
				this.removeThemeCSS(id);
			}

			this.save();
			return true;
		}

		return false;
	}

	/**
	 * Get a specific theme by ID
	 */
	getTheme(id: string): CustomTheme | undefined {
		return this.themes.find((t) => t.id === id);
	}

	/**
	 * Get all custom themes
	 */
	getAllThemes(): CustomTheme[] {
		return [...this.themes];
	}

	/**
	 * Inject theme CSS into the document
	 */
	injectThemeCSS(id: string): boolean {
		const theme = this.getTheme(id);
		if (!theme) return false;

		// Remove previously injected theme
		if (this.injectedThemeId) {
			this.removeThemeCSS(this.injectedThemeId);
		}

		// Create and inject style element
		const styleId = `custom-theme-${id}`;
		let styleElement = document.getElementById(styleId) as HTMLStyleElement;

		if (!styleElement) {
			styleElement = document.createElement('style');
			styleElement.id = styleId;
			document.head.appendChild(styleElement);
		}

		styleElement.textContent = theme.css;

		// Extract theme name from CSS to set data-theme attribute
		const themeNameMatch = theme.css.match(/\[data-theme=['"]([^'"]+)['"]\]/);
		const dataThemeName = themeNameMatch?.[1] || id;

		document.documentElement.setAttribute('data-theme', dataThemeName);

		this.injectedThemeId = id;
		return true;
	}

	/**
	 * Remove injected theme CSS from the document
	 */
	removeThemeCSS(id: string): void {
		const styleId = `custom-theme-${id}`;
		const styleElement = document.getElementById(styleId);

		if (styleElement) {
			styleElement.remove();
		}

		if (this.injectedThemeId === id) {
			this.injectedThemeId = null;
		}
	}

	/**
	 * Remove all injected custom theme CSS
	 */
	removeAllInjectedCSS(): void {
		if (this.injectedThemeId) {
			this.removeThemeCSS(this.injectedThemeId);
		}
	}

	/**
	 * Export a theme as a downloadable CSS file
	 */
	exportTheme(id: string): void {
		const theme = this.getTheme(id);
		if (!theme) return;

		const blob = new Blob([theme.css], { type: 'text/css' });
		const url = URL.createObjectURL(blob);
		const a = document.createElement('a');
		a.href = url;
		a.download = `${theme.name.replace(/[^a-z0-9]/gi, '-').toLowerCase()}.css`;
		document.body.appendChild(a);
		a.click();
		document.body.removeChild(a);
		URL.revokeObjectURL(url);
	}

	/**
	 * Update theme name
	 */
	updateThemeName(id: string, newName: string): boolean {
		const theme = this.themes.find((t) => t.id === id);
		if (!theme) return false;

		// Check for duplicate name
		if (this.themes.some((t) => t.id !== id && t.name.toLowerCase() === newName.toLowerCase())) {
			return false;
		}

		theme.name = newName;
		this.save();
		return true;
	}

	/**
	 * Clear all custom themes
	 */
	clearAll(): void {
		this.removeAllInjectedCSS();
		this.themes = [];
		this.save();
	}

	// Getters
	get count(): number {
		return this.themes.length;
	}

	get isInitialized(): boolean {
		return this.initialized;
	}

	get canAddMore(): boolean {
		return this.themes.length < MAX_THEMES;
	}

	get currentInjectedId(): string | null {
		return this.injectedThemeId;
	}
}

// Export singleton instance
export const customThemesStore = new CustomThemesStore();
