// const BASE_URL = '';
import type { ErrorStack } from '../types';
import { kernelManagerApi } from './kernel_manager';
import { resolveApi } from './resolve';
import { storageApi } from './storage';

export async function fetchJson<T>(
	path: string,
	init: RequestInit = {},
	query: URLSearchParams = new URLSearchParams()
): Promise<T> {
	if (query.toString()) {
		path += `?${query.toString()}`;
	}
	let response = await fetch(path, {
		headers: {
			'Content-Type': 'application/json',
			...init.headers
		},
		...init
	});
	response = await catchAndThrowHttpError(response);
	return (await response.json()) as T;
}

export async function fetchQuery<T>(
	path: string,
	query: URLSearchParams = new URLSearchParams(),
	init: RequestInit = {}
): Promise<T> {
	if (query.toString()) {
		path += `?${query.toString()}`;
	}
	let response = await fetch(path, init);
	response = await catchAndThrowHttpError(response);
	return (await response.json()) as T;
}
export async function fetchText(
	url: string,
	init: RequestInit = {},
	query: URLSearchParams = new URLSearchParams()
): Promise<string> {
	let urlWithParams = url;
	if (query.toString()) {
		urlWithParams += `?${query.toString()}`;
	}
	const response = await fetch(urlWithParams, {
		...init
	});
	const checkedResponse = await catchAndThrowHttpError(response);
	return await checkedResponse.text();
}

async function catchAndThrowHttpError(response: Response): Promise<Response> {
	if (response.status === 500) {
		const errorStack = await response.json();
		throw new InternalError(errorStack);
	} else if (!response.ok) {
		const errorDetail = await response.text();
		throw new Error(`HTTP ${response.status}: ${errorDetail}`);
	} else {
		return response;
	}
}

export const api = {
	kernelManager: kernelManagerApi,
	resolve: resolveApi,
	storage: storageApi
};

export class InternalError extends Error {
	stacks: ErrorStack;
	constructor(error: ErrorStack) {
		super('Http Internal Error');
		this.stacks = error;
	}
}
