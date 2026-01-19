import { HttpErrorType, type HttpError, type ErrorStack } from '$lib/types/http-error';

/**
 * Parse ErrorStack from x-error-stack header
 */
function parseErrorStack(headerValue: string): ErrorStack | undefined {
	try {
		const parsed = JSON.parse(headerValue);
		// Validate the structure
		if (typeof parsed === 'object' && parsed !== null && typeof parsed.message === 'string') {
			return {
				message: parsed.message,
				stack: typeof parsed.stack === 'string' ? parsed.stack : undefined,
				code: typeof parsed.code === 'string' ? parsed.code : undefined,
				details:
					typeof parsed.details === 'object' && parsed.details !== null
						? parsed.details
						: undefined
			};
		}
	} catch (e) {
		// Invalid JSON, return undefined
		console.warn('Failed to parse x-error-stack header:', e);
	}
	return undefined;
}

/**
 * Parse a Response object into HttpError
 * Automatically detects if it's an internal error with error stack
 */
export async function parseResponseError(response: Response): Promise<HttpError> {
	const url = response.url;
	const status = response.status;
	const statusText = response.statusText;

	// Check for x-error-stack header (internal server error with stack trace)
	const errorStackHeader = response.headers.get('x-error-stack');
	if (errorStackHeader) {
		const errorStack = parseErrorStack(errorStackHeader);
		if (errorStack) {
			return {
				type: HttpErrorType.INTERNAL_ERROR,
				status,
				statusText,
				message: errorStack.message,
				url,
				errorStack
			};
		}
	}

	// Try to parse response body for additional error information
	let bodyMessage: string | undefined;
	try {
		const contentType = response.headers.get('content-type');
		if (contentType?.includes('application/json')) {
			const body = await response.json();
			// Common error response formats
			bodyMessage =
				body.error?.message || body.message || body.error || JSON.stringify(body);
		} else if (contentType?.includes('text/')) {
			bodyMessage = await response.text();
		}
	} catch (e) {
		// Failed to parse body, ignore
	}

	// Generic HTTP error
	const message = bodyMessage || `HTTP ${status}: ${statusText}`;

	return {
		type: HttpErrorType.HTTP_ERROR,
		status,
		statusText,
		message,
		url
	};
}

/**
 * Parse a fetch error (network error, timeout, etc.) into HttpError
 */
export function parseNetworkError(error: Error, url?: string): HttpError {
	return {
		type: HttpErrorType.NETWORK_ERROR,
		message: error.message || 'Network request failed',
		url,
		originalError: error
	};
}

/**
 * Universal fetch wrapper that automatically parses errors into HttpError
 * Usage:
 *   try {
 *     const data = await fetchWithErrorHandling('/api/data');
 *   } catch (error) {
 *     // error is HttpError
 *     showError(error);
 *   }
 */
export async function fetchWithErrorHandling(
	input: RequestInfo | URL,
	init?: RequestInit
): Promise<Response> {
	const url = typeof input === 'string' ? input : input instanceof URL ? input.href : input.url;

	try {
		const response = await fetch(input, init);

		if (!response.ok) {
			throw await parseResponseError(response);
		}

		return response;
	} catch (error) {
		// If it's already an HttpError, re-throw it
		if (error && typeof error === 'object' && 'type' in error) {
			throw error;
		}

		// Otherwise, it's a network error
		throw parseNetworkError(error as Error, url);
	}
}

/**
 * Check if an error is an HttpError
 */
export function isHttpError(error: unknown): error is HttpError {
	return (
		error !== null &&
		typeof error === 'object' &&
		'type' in error &&
		Object.values(HttpErrorType).includes((error as HttpError).type)
	);
}

/**
 * Get a user-friendly error message from HttpError
 * Falls back to generic message if error is not HttpError
 */
export function getErrorMessage(error: unknown): string {
	if (isHttpError(error)) {
		return error.message;
	}

	if (error instanceof Error) {
		return error.message;
	}

	return String(error);
}

/**
 * Get HTTP status code from error if available
 */
export function getErrorStatus(error: unknown): number | undefined {
	if (isHttpError(error)) {
		return error.status;
	}
	return undefined;
}

/**
 * Check if error is retryable (network errors are usually retryable)
 */
export function isRetryableError(error: unknown): boolean {
	if (!isHttpError(error)) {
		return false;
	}

	// Network errors are retryable
	if (error.type === HttpErrorType.NETWORK_ERROR) {
		return true;
	}

	// Some HTTP status codes are retryable
	if (error.status) {
		// 408 Request Timeout, 429 Too Many Requests, 500-599 Server Errors
		return error.status === 408 || error.status === 429 || error.status >= 500;
	}

	return false;
}
