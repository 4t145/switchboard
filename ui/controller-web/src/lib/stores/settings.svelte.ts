/**
 * Settings Store - Manages user preferences
 * Uses Svelte 5 runes for reactive state management
 */

// Type definitions
export type ThemeName = 'vintage' | 'modern' | 'rocket' | 'seafoam' | 'skeleton' | 'sahara';
export type LanguageCode = 'zh' | 'en';

export interface UserSettings {
	theme: ThemeName;
	darkMode: boolean;
	language: LanguageCode;
}

// Storage key
const STORAGE_KEY = 'switchboard-settings';
const STORAGE_VERSION = 1;

interface StoredSettings extends UserSettings {
	version: number;
}

// Default settings
const DEFAULT_SETTINGS: UserSettings = {
	theme: 'vintage',
	darkMode: false,
	language: 'zh'
};

// Theme metadata for UI display
export const THEMES: Record<ThemeName, { name: string; colors: string[] }> = {
	vintage: { name: 'Vintage', colors: ['#7C3AED', '#10B981', '#3B82F6'] },
	modern: { name: 'Modern', colors: ['#0EA5E9', '#64748B', '#F59E0B'] },
	rocket: { name: 'Rocket', colors: ['#EF4444', '#F97316', '#EAB308'] },
	seafoam: { name: 'Seafoam', colors: ['#14B8A6', '#06B6D4', '#0891B2'] },
	skeleton: { name: 'Skeleton', colors: ['#64748B', '#94A3B8', '#CBD5E1'] },
	sahara: { name: 'Sahara', colors: ['#F59E0B', '#D97706', '#B45309'] }
};

/**
 * Settings Store Class
 * Manages user preferences with automatic persistence
 */
class SettingsStore {
	private settings = $state<UserSettings>({ ...DEFAULT_SETTINGS });
	private initialized = $state(false);

	constructor() {
		// Load settings on client side only
		if (typeof window !== 'undefined') {
			this.load();
			this.initialized = true;
		}
	}

	/**
	 * Load settings from localStorage
	 */
	load(): void {
		if (typeof window === 'undefined') return;

		try {
			const stored = localStorage.getItem(STORAGE_KEY);
			if (stored) {
				const parsed: StoredSettings = JSON.parse(stored);

				// Validate version and data
				if (parsed.version === STORAGE_VERSION) {
					this.settings = {
						theme: parsed.theme || DEFAULT_SETTINGS.theme,
						darkMode: parsed.darkMode ?? DEFAULT_SETTINGS.darkMode,
						language: parsed.language || DEFAULT_SETTINGS.language
					};
				}
			}
		} catch (error) {
			console.error('Failed to load settings from localStorage:', error);
			this.settings = { ...DEFAULT_SETTINGS };
		}

		// Apply settings immediately after loading
		this.apply();
	}

	/**
	 * Save settings to localStorage
	 */
	private save(): void {
		if (typeof window === 'undefined') return;

		try {
			const toStore: StoredSettings = {
				version: STORAGE_VERSION,
				...this.settings
			};
			localStorage.setItem(STORAGE_KEY, JSON.stringify(toStore));
		} catch (error) {
			console.error('Failed to save settings to localStorage:', error);
		}
	}

	/**
	 * Apply current settings to DOM
	 */
	apply(): void {
		if (typeof window === 'undefined') return;

		// Apply theme
		document.documentElement.setAttribute('data-theme', this.settings.theme);

		// Apply dark mode
		if (this.settings.darkMode) {
			document.documentElement.classList.add('dark');
		} else {
			document.documentElement.classList.remove('dark');
		}
	}

	/**
	 * Reset all settings to defaults
	 */
	reset(): void {
		this.settings = { ...DEFAULT_SETTINGS };
		this.save();
		this.apply();
	}

	// Getters
	get theme(): ThemeName {
		return this.settings.theme;
	}

	get darkMode(): boolean {
		return this.settings.darkMode;
	}

	get language(): LanguageCode {
		return this.settings.language;
	}

	get isInitialized(): boolean {
		return this.initialized;
	}

	// Setters with auto-save
	set theme(value: ThemeName) {
		this.settings.theme = value;
		this.save();
		this.apply();
	}

	set darkMode(value: boolean) {
		this.settings.darkMode = value;
		this.save();
		this.apply();
	}

	set language(value: LanguageCode) {
		this.settings.language = value;
		this.save();
		// Language application will be handled by Paraglide integration
	}

	/**
	 * Get all settings as plain object
	 */
	getAll(): UserSettings {
		return { ...this.settings };
	}
}

// Export singleton instance
export const settingsStore = new SettingsStore();
