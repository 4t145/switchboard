const BASE_URL = '';
import { kernelManagerApi } from './kernel_manager';

export async function fetchJson<T>(path: string, init: RequestInit = {}): Promise<T> {
	const url = path.startsWith('http') ? path : `${BASE_URL.replace(/\/$/, '')}${path}`;
	const response = await fetch(url, {
		headers: {
			'Content-Type': 'application/json',
			...init.headers,
		},
		...init,
	});
	const text = await response.text();
	const parsed = text ? (JSON.parse(text) as T) : (undefined as unknown as T);
	if (!response.ok) {
		const errorDetail = typeof parsed === 'object' ? parsed : { message: text || response.statusText };
		throw new Error(`HTTP ${response.status}: ${JSON.stringify(errorDetail)}`);
	}
	return parsed;
}


export const api =  {
	kernelManager: kernelManagerApi
}