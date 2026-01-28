export type HttpVersion = 'auto' | 'http1' | 'http2';

export interface ServerConfig {
	version: HttpVersion;
}

export interface HttpConfig<Cfg = unknown> {
	flow: FlowConfig<Cfg>;
	server?: ServerConfig;
}

export interface FlowConfig<Cfg = unknown> {
	entrypoint: NodeTarget;
	nodes: Record<string, InstanceDataWithoutType<Cfg>>;
	filters: Record<string, InstanceDataWithoutType<Cfg>>;
	options?: FlowOptions;
}

export interface FlowOptions {
	max_loop?: number;
}

export type InstanceType = 'node' | 'filter';

export type NodeId = string;

export type NodePort = string;

export type NodeTarget = string;
export type NodeTargetObject = {
	nodeId: string;
	port: string;
}
export const NodeTarget = {
	parse(target: NodeTarget): NodeTargetObject {
		if (!target.includes(':')) {
			return { nodeId: target, port: '$default' };
		} else {
			const [nodeId, port] = target.split(':', 2);
			return { nodeId, port };
		}
	}
};

export interface InstanceData<Cfg = unknown> {
	name?: string;
	class: string;
	type: InstanceType;
	config: Cfg;
}

export interface InstanceDataWithoutType<Cfg = unknown> {
	name?: string;
	class: string;
	config: Cfg;
}

export interface NodeInput {
	filters?: FilterReference[];
}

export interface NodeOutput {
	filters?: FilterReference[];
	target: NodeTarget;
}

export interface NodeInterface {
	inputs?: Record<NodePort, NodeInput>;
	outputs?: Record<NodePort, NodeOutput>;
}

export type FilterId = string;

export interface FilterReference {
	id: FilterId;
}

export interface ClassId {
	namespace?: string;
	name: string;
}

export interface ClassMeta {
	version: string;
	description?: string;
	author?: string;
	license?: string;
	repository?: string;
	homepage?: string;
}

export interface ClassData {
	id: ClassId;
	meta: ClassMeta;
	instance_type: InstanceType;
}

export type WithOutputs<C> = {
	output?: Record<NodePort, NodeOutput>;
} & C;
