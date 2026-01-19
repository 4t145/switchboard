# HTTP Error Handling System

A comprehensive error handling system for HTTP requests with visual error display components.

## Overview

This system provides:
- **Type-safe error parsing** for network, HTTP, and internal server errors
- **Visual error display component** with expandable details
- **Automatic error detection** from fetch responses
- **i18n support** for error messages

## Files

### Core Files
- `src/lib/types/http-error.ts` - TypeScript type definitions
- `src/lib/utils/http-error-parser.ts` - Error parsing utilities
- `src/lib/components/error-display.svelte` - Visual error display component

### Demo
- `src/routes/admin/error-demo/+page.svelte` - Interactive demo page

## Error Types

### 1. Network Error
Connection failures, timeouts, no internet connection.

```typescript
{
  type: HttpErrorType.NETWORK_ERROR,
  message: 'Failed to connect to server',
  url: 'https://api.example.com/data',
  originalError: Error
}
```

### 2. HTTP Error
Non-2xx HTTP status codes (404, 401, 500, etc.)

```typescript
{
  type: HttpErrorType.HTTP_ERROR,
  status: 404,
  statusText: 'Not Found',
  message: 'The requested resource was not found',
  url: 'https://api.example.com/users/123'
}
```

### 3. Internal Error
Server error with detailed stack trace via `x-error-stack` header.

```typescript
{
  type: HttpErrorType.INTERNAL_ERROR,
  status: 500,
  statusText: 'Internal Server Error',
  message: 'Database connection pool exhausted',
  url: 'https://api.example.com/data',
  errorStack: {
    message: 'Database connection pool exhausted',
    code: 'DB_POOL_EXHAUSTED',
    stack: '...',
    details: { poolSize: 10, activeConnections: 10 }
  }
}
```

## Usage

### Quick Start (Recommended)

Use `fetchWithErrorHandling` for automatic error parsing:

```svelte
<script>
  import ErrorDisplay from '$lib/components/error-display.svelte';
  import { fetchWithErrorHandling } from '$lib/utils/http-error-parser';

  let error = $state(null);
  let data = $state(null);

  async function loadData() {
    error = null;
    try {
      const response = await fetchWithErrorHandling('/api/data');
      data = await response.json();
    } catch (err) {
      error = err; // Already an HttpError
    }
  }
</script>

{#if error}
  <ErrorDisplay 
    {error} 
    onRetry={loadData} 
    onDismiss={() => error = null}
    dismissible 
  />
{/if}
```

### Manual Error Parsing

For custom fetch logic:

```typescript
import { parseResponseError, parseNetworkError } from '$lib/utils/http-error-parser';

async function customFetch(url: string) {
  try {
    const response = await fetch(url);
    
    if (!response.ok) {
      throw await parseResponseError(response);
    }
    
    return response.json();
  } catch (err) {
    // If it's not already an HttpError, treat as network error
    if (err instanceof Error && !('type' in err)) {
      throw parseNetworkError(err, url);
    }
    throw err;
  }
}
```

### Backend Integration

To send internal errors with stack traces, add `x-error-stack` header:

```rust
// Rust example with Axum
use axum::http::{HeaderMap, HeaderValue};
use serde_json::json;

fn error_response(error: &AppError) -> Response {
    let error_stack = json!({
        "message": error.message,
        "code": error.code,
        "stack": error.backtrace.to_string(),
        "details": error.details
    });
    
    let mut headers = HeaderMap::new();
    headers.insert(
        "x-error-stack",
        HeaderValue::from_str(&error_stack.to_string()).unwrap()
    );
    
    (StatusCode::INTERNAL_SERVER_ERROR, headers, "").into_response()
}
```

## Component API

### ErrorDisplay Props

```typescript
interface Props {
  error: HttpError;           // Required: The error to display
  onRetry?: () => void;       // Optional: Retry callback (shows retry button if retryable)
  onDismiss?: () => void;     // Optional: Dismiss callback
  dismissible?: boolean;      // Optional: Show dismiss button (default: false)
  showDetails?: boolean;      // Optional: Show URL and stack (default: true)
  class?: string;             // Optional: Additional CSS classes
}
```

### Visual Features

- **Color-coded by error type**:
  - Network errors: Yellow/Warning
  - HTTP errors: Red/Error
  - Internal errors: Red/Error with expandable stack trace

- **Automatic retry detection**: Shows retry button only for retryable errors (network errors, 408, 429, 5xx)

- **Expandable error details**: For internal errors with stack traces

- **Responsive design**: Works on mobile and desktop

## Utility Functions

```typescript
// Check if error is HttpError
isHttpError(error: unknown): boolean

// Get user-friendly error message
getErrorMessage(error: unknown): string

// Get HTTP status code if available
getErrorStatus(error: unknown): number | undefined

// Check if error is retryable
isRetryableError(error: unknown): boolean
```

## Examples

### Basic Error Display
```svelte
<ErrorDisplay {error} />
```

### With Retry and Dismiss
```svelte
<ErrorDisplay 
  {error} 
  onRetry={handleRetry}
  onDismiss={handleDismiss}
  dismissible
/>
```

### Custom Styling
```svelte
<ErrorDisplay {error} class="my-4 max-w-2xl" />
```

### Hide Details
```svelte
<ErrorDisplay {error} showDetails={false} />
```

## Demo

Visit `/admin/error-demo` in the application to see:
- Interactive examples of all error types
- Usage code samples
- Component API documentation
- Live error display testing

## Translations

Error messages support i18n. Add translations in `messages/{lang}.json`:

```json
{
  "error_type_network": "Network Error",
  "error_type_http": "HTTP Error",
  "error_type_internal": "Internal Error",
  "error_details_toggle": "Show Details",
  "error_retry": "Retry",
  "error_dismiss": "Dismiss"
}
```

## Best Practices

1. **Always use `fetchWithErrorHandling`** for automatic error parsing
2. **Show retry button** for user-recoverable errors
3. **Log errors** to your monitoring system
4. **Provide context** in error messages
5. **Test error states** using the demo page
6. **Handle loading states** alongside error states

## Error Recovery Patterns

### With Retry Logic
```svelte
<script>
  let retryCount = $state(0);
  let maxRetries = 3;

  async function loadData() {
    try {
      const response = await fetchWithErrorHandling('/api/data');
      return await response.json();
    } catch (err) {
      if (retryCount < maxRetries && isRetryableError(err)) {
        retryCount++;
        setTimeout(loadData, 1000 * retryCount); // Exponential backoff
      } else {
        error = err;
      }
    }
  }
</script>
```

### Global Error Handler
```typescript
// src/lib/stores/errors.svelte.ts
export function createErrorStore() {
  let errors = $state<HttpError[]>([]);
  
  return {
    get errors() { return errors; },
    add: (error: HttpError) => errors.push(error),
    remove: (index: number) => errors.splice(index, 1),
    clear: () => errors = []
  };
}
```

## License

Part of the Switchboard project.
