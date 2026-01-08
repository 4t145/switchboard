import { getContext } from 'svelte';
import { getClass, type Theme, type ThemeName, type ThemeOptions } from './index';
import { retro } from './instances/index.js';
interface ThemeContext {
	currentTheme: Theme;
	themeName: ThemeName;
	options: ThemeOptions;
	setTheme: (name: ThemeName) => void;
	// toggleTheme: () => void;
}

export function useTheme(): ThemeContext {
	const context = getContext<ThemeContext>('theme');
	if (!context) {
		throw new Error('useTheme must be used within a ThemeProvider');
	}
	return context;
}

export function defaultThemeContext(): ThemeContext {
	const defaultTheme = retro;
	return {
		currentTheme: defaultTheme,
		themeName: defaultTheme.name,
		options: { darkMode: false },
		setTheme: (name: ThemeName) => {
			console.warn(`setTheme called with ${name}, but no ThemeProvider is set.`);
		}
	};
}
export function getThemeClass(context: ThemeContext, componentName: ThemeName): string {
	return getClass(context.currentTheme, componentName, context.options);
}
