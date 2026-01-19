/**
 * Settings Store - Manages user preferences
 * Uses Svelte 5 runes for reactive state management
 */

import { setLocale } from '$lib/paraglide/runtime';
import { customThemesStore } from './custom-themes.svelte';

// Type definitions
export type PresetThemeName = 'vintage' | 'modern' | 'rocket' | 'seafoam' | 'cerberus' | 'sahara' | 'astro';
export type CustomThemeId = `custom:${string}`;
export type ThemeName = PresetThemeName | CustomThemeId | string; // string for compatibility
export type LanguageCode = 'zh' | 'en';
export type ColorMode = 'light' | 'dark' | 'auto';

export interface UserSettings {
	theme: string; // Changed from ThemeName to string for flexibility
	colorMode: ColorMode;
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
	colorMode: 'auto',
	language: 'zh'
};

// Theme metadata for UI display (preset themes only)
export const PRESET_THEMES: Record<PresetThemeName, { name: string; colors: string[] }> = {
	vintage: { name: 'Vintage', colors: ['#7C3AED', '#10B981', '#3B82F6'] },
	modern: { name: 'Modern', colors: ['#0EA5E9', '#64748B', '#F59E0B'] },
	rocket: { name: 'Rocket', colors: ['#EF4444', '#F97316', '#EAB308'] },
	seafoam: { name: 'Seafoam', colors: ['#14B8A6', '#06B6D4', '#0891B2'] },
	cerberus: { name: 'Cerberus', colors: ['#8B5CF6', '#EC4899', '#F59E0B'] },
	sahara: { name: 'Sahara', colors: ['#F59E0B', '#D97706', '#B45309'] },
	astro: { name: 'Astro', colors: ['#b3514b', '#e6d7a8', '#5b7baa'] }
};

// For backwards compatibility
export const THEMES = PRESET_THEMES;

/**
 * Settings Store Class
 * Manages user preferences with automatic persistence
 */
class SettingsStore {
	private settings = $state<UserSettings>({ ...DEFAULT_SETTINGS });
	private initialized = $state(false);
	private mediaQuery: MediaQueryList | null = null;

	constructor() {
		// Load settings on client side only
		if (typeof window !== 'undefined') {
			this.load();
			this.initialized = true;
			this.setupSystemPreferenceListener();
		}
	}

	/**
	 * Setup system preference listener for auto mode
	 */
	private setupSystemPreferenceListener(): void {
		if (typeof window === 'undefined') return;

		this.mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
		
		// Apply immediately if in auto mode
		if (this.settings.colorMode === 'auto') {
			this.apply();
		}

		// Listen for system preference changes
		const handler = () => {
			if (this.settings.colorMode === 'auto') {
				this.apply();
			}
		};
		
		this.mediaQuery.addEventListener('change', handler);
	}

	/**
	 * Detect browser language preference
	 */
	private detectBrowserLanguage(): LanguageCode {
		if (typeof navigator === 'undefined') return 'zh';
		
		const browserLang = navigator.language || (navigator as any).userLanguage;
		
		// Check if browser language starts with 'zh' (Chinese)
		if (browserLang.toLowerCase().startsWith('zh')) {
			return 'zh';
		}
		
		// Default to English for all other languages
		return 'en';
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
						colorMode: parsed.colorMode || this.migrateOldDarkMode(parsed),
						language: parsed.language || this.detectBrowserLanguage()
					};
				} else {
					// New user or version upgrade, use browser language
					this.settings = {
						...DEFAULT_SETTINGS,
						language: this.detectBrowserLanguage()
					};
				}
			} else {
				// First time user, detect browser language
				this.settings = {
					...DEFAULT_SETTINGS,
					language: this.detectBrowserLanguage()
				};
			}
		} catch (error) {
			console.error('Failed to load settings from localStorage:', error);
			this.settings = {
				...DEFAULT_SETTINGS,
				language: this.detectBrowserLanguage()
			};
		}

		// Apply settings immediately after loading
		this.apply();
	}

	/**
	 * Migrate old darkMode boolean to new colorMode
	 */
	private migrateOldDarkMode(parsed: any): ColorMode {
		if ('darkMode' in parsed) {
			return parsed.darkMode ? 'dark' : 'light';
		}
		return DEFAULT_SETTINGS.colorMode;
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

		// Check if this is a custom theme
		if (this.settings.theme.startsWith('custom:')) {
			const themeId = this.settings.theme.slice(7); // Remove 'custom:' prefix
			const success = customThemesStore.injectThemeCSS(themeId);
			
			if (!success) {
				// If custom theme not found, fall back to default
				console.warn(`Custom theme ${themeId} not found, falling back to default`);
				this.settings.theme = DEFAULT_SETTINGS.theme;
				this.save();
				document.documentElement.setAttribute('data-theme', this.settings.theme);
			}
		} else {
			// Remove any injected custom theme CSS
			customThemesStore.removeAllInjectedCSS();
			
			// Apply preset theme
			document.documentElement.setAttribute('data-theme', this.settings.theme);
		}

		// Apply color mode using data-mode attribute (Skeleton UI recommended approach)
		const mode = this.resolveColorMode();
		document.documentElement.setAttribute('data-mode', mode);

		// Apply language
		setLocale(this.settings.language);
	}

	/**
	 * Resolve the actual color mode based on user preference and system settings
	 */
	private resolveColorMode(): 'light' | 'dark' {
		if (this.settings.colorMode === 'auto') {
			// Use system preference
			if (typeof window !== 'undefined' && window.matchMedia) {
				return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
			}
			return 'light'; // Fallback
		}
		return this.settings.colorMode;
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
	get theme(): string {
		return this.settings.theme;
	}

	get colorMode(): ColorMode {
		return this.settings.colorMode;
	}

	get language(): LanguageCode {
		return this.settings.language;
	}

	get isInitialized(): boolean {
		return this.initialized;
	}

	get isCustomTheme(): boolean {
		return this.settings.theme.startsWith('custom:');
	}

	get isDarkMode(): boolean {
		return this.resolveColorMode() === 'dark';
	}

	// Setters with auto-save
	set theme(value: string) {
		this.settings.theme = value;
		this.save();
		this.apply();
	}

	set colorMode(value: ColorMode) {
		this.settings.colorMode = value;
		this.save();
		this.apply();
	}

	set language(value: LanguageCode) {
		this.settings.language = value;
		this.save();
		setLocale(value); // Update Paraglide language
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
