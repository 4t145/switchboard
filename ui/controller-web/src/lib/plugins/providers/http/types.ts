/**
 * TypeScript types for HTTP service configuration
 * Based on Rust types from crates/model/src/services/http.rs
 */

export type HttpVersion = 'auto' | 'http1' | 'http2';

export interface ServerConfig {
	version: HttpVersion;
}

export interface HttpConfig {
	flow: FlowConfig;
	server?: ServerConfig;
}

export interface FlowConfig {
	entrypoint: NodeTarget;
	instances?: Record<string, InstanceData>;
	nodes?: Record<string, InstanceDataWithoutType>;
	filters?: Record<string, InstanceDataWithoutType>;
	options?: FlowOptions;
}

export interface FlowOptions {
	max_loop?: number;
}

export interface NodeTarget {
	node: string;
}

export type InstanceType = 'node' | 'filter';

export interface InstanceData {
	name?: string;
	class: string;
	type: InstanceType;
	config: any;
}

export interface InstanceDataWithoutType {
	name?: string;
	class: string;
	config: any;
}
