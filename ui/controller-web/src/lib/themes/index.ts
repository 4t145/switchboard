type ClassLike = string | ((options: ThemeOptions) => string);

function intoClassString(classLike: ClassLike, options: ThemeOptions): string {
	if (typeof classLike === 'string') {
		return classLike;
	} else {
		return classLike(options);
	}
}

export type ThemeName = 'light' | 'dark' | string;
export type Theme = {
	name: ThemeName;
	styles: Record<ThemeName, ClassLike>;
};

export type ThemeOptions = {
	darkMode: boolean;
};

export function getClass(theme: Theme, componentName: ThemeName, options: ThemeOptions): string {
	return intoClassString(theme.styles[componentName] || '', options);
}
