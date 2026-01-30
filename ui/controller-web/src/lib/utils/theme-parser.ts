/**
 * Theme Parser - Utilities for parsing and validating Skeleton theme CSS files
 */

export interface ParsedTheme {
	name: string;
	css: string;
	colors: string[];
	isValid: boolean;
	error?: string;
}

const MAX_FILE_SIZE = 500 * 1024; // 500KB
const COLOR_SHADES = ['50', '100', '200', '300', '400', '500', '600', '700', '800', '900', '950'];

/**
 * Parse and validate a Skeleton theme CSS file
 */
export function parseThemeCSS(cssContent: string, fileName: string = ''): ParsedTheme {
	// Check file size (approximate, since we have the content as string)
	if (new Blob([cssContent]).size > MAX_FILE_SIZE) {
		return {
			name: '',
			css: '',
			colors: [],
			isValid: false,
			error: 'File too large (max 500KB)'
		};
	}

	// Extract theme name from [data-theme='xxx'] selector
	const themeNameMatch = cssContent.match(/\[data-theme=['"]([^'"]+)['"]\]/);
	let themeName = themeNameMatch?.[1] || '';

	// If no theme name found in CSS, try to use filename
	if (!themeName && fileName) {
		themeName = fileName.replace(/\.css$/i, '').replace(/[-_]/g, ' ');
		themeName = themeName.charAt(0).toUpperCase() + themeName.slice(1);
	}

	if (!themeName) {
		return {
			name: '',
			css: '',
			colors: [],
			isValid: false,
			error: 'Invalid theme format: no [data-theme] selector found'
		};
	}

	// Validate CSS structure - should contain Skeleton theme variables
	const hasSkeletonVars =
		cssContent.includes('--color-primary-') ||
		cssContent.includes('--base-font-') ||
		cssContent.includes('--heading-font-');

	if (!hasSkeletonVars) {
		return {
			name: themeName,
			css: '',
			colors: [],
			isValid: false,
			error: 'Invalid theme format: missing Skeleton theme variables'
		};
	}

	// Sanitize CSS (basic safety check)
	const sanitized = sanitizeCSS(cssContent);

	// Extract preview colors
	const colors = extractColors(sanitized);

	return {
		name: themeName,
		css: sanitized,
		colors,
		isValid: true
	};
}

/**
 * Extract color values for preview
 * Extracts primary colors at different shades
 */
export function extractColors(css: string): string[] {
	const colors: string[] = [];
	const shadesToExtract = ['500', '600', '400', '700', '300']; // Most representative shades

	for (const shade of shadesToExtract) {
		const regex = new RegExp(`--color-primary-${shade}:\\s*([^;]+);`, 'i');
		const match = css.match(regex);

		if (match && match[1]) {
			const colorValue = match[1].trim();
			// Convert oklch to hex approximation or use as-is
			const displayColor = convertToDisplayColor(colorValue);
			colors.push(displayColor);
		}
	}

	// Fallback colors if not enough found
	while (colors.length < 3) {
		colors.push('#9CA3AF');
	}

	return colors.slice(0, 5); // Return up to 5 colors for preview
}

/**
 * Convert oklch/other color formats to displayable hex or rgb
 * For simplicity, we'll just use the color as-is and let CSS handle it
 */
function convertToDisplayColor(colorValue: string): string {
	// If it's already hex or rgb, use it
	if (colorValue.startsWith('#') || colorValue.startsWith('rgb')) {
		return colorValue;
	}

	// For oklch and other formats, we'll return a CSS variable reference
	// The browser will handle the actual color display
	return colorValue;
}

/**
 * Sanitize CSS content to prevent malicious code injection
 */
export function sanitizeCSS(css: string): string {
	let sanitized = css;

	// Remove any script tags (shouldn't be in CSS, but safety first)
	sanitized = sanitized.replace(/<script\b[^<]*(?:(?!<\/script>)<[^<]*)*<\/script>/gi, '');

	// Remove javascript: URLs
	sanitized = sanitized.replace(/javascript:/gi, '');

	// Remove @import statements that try to load external resources
	sanitized = sanitized.replace(/@import\s+url\([^)]+\)/gi, '');

	// Remove any eval or expression
	sanitized = sanitized.replace(/expression\s*\(/gi, '');
	sanitized = sanitized.replace(/eval\s*\(/gi, '');

	// Only allow [data-theme='xxx'] selectors and standard CSS
	// This is a basic check - in production, you might want more sophisticated validation

	return sanitized;
}

/**
 * Validate that a theme name is unique and safe
 */
export function validateThemeName(
	name: string,
	existingNames: string[]
): { valid: boolean; error?: string } {
	if (!name || name.trim().length === 0) {
		return { valid: false, error: 'Theme name cannot be empty' };
	}

	if (name.length > 50) {
		return { valid: false, error: 'Theme name too long (max 50 characters)' };
	}

	// Check for unsafe characters
	if (!/^[a-zA-Z0-9\s\-_]+$/.test(name)) {
		return { valid: false, error: 'Theme name contains invalid characters' };
	}

	// Check for duplicates (case-insensitive)
	if (existingNames.some((existing) => existing.toLowerCase() === name.toLowerCase())) {
		return { valid: false, error: 'A theme with this name already exists' };
	}

	return { valid: true };
}

/**
 * Generate a unique ID for a custom theme
 */
export function generateThemeId(): string {
	return `custom_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
}

/**
 * Validate file type
 */
export function validateFileType(file: File): boolean {
	return (file.name.toLowerCase().endsWith('.css') && file.type === 'text/css') || file.type === '';
}
