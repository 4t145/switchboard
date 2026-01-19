/**
 * Error Types for Network Requests
 */

/**
 * Error stack information from x-error-stack header
 */
export interface ErrorStack {
	message: string;
	stack?: string;
	code?: string;
	details?: Record<string, unknown>;
}

/**
 * HTTP Error types
 */
export enum HttpErrorType {
	/** Network request failed (fetch failed, timeout, etc.) */
	NETWORK_ERROR = 'NETWORK_ERROR',
	/** HTTP status code indicates error (4xx, 5xx) */
	HTTP_ERROR = 'HTTP_ERROR',
	/** 500 Internal Server Error with error stack */
	INTERNAL_ERROR = 'INTERNAL_ERROR'
}

/**
 * Parsed HTTP Error
 */
export interface HttpError {
	type: HttpErrorType;
	status?: number;
	statusText?: string;
	message: string;
	url?: string;
	errorStack?: ErrorStack;
	originalError?: Error;
}
