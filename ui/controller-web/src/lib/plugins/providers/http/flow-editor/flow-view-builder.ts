import { getHttpClassEditorPlugin } from '$lib/plugins/registry';
import { isLinkValue } from '$lib/utils/link-parser';
import {
	type FlowConfig,
	type InstanceDataWithoutType,
	type NodeId,
	type NodePort,
	type NodeTargetObject
} from '../types';
import { api } from '$lib/api/routes';
import type { InputInfo, OutputInfo } from '$lib/plugins/types';

function getOutputs(nodeId: string, node: InstanceDataWithoutType<unknown>): OutputInfo[] {
	const plugin = getHttpClassEditorPlugin(node.class);
	try {
		if (!plugin) {
			throw new Error(`Plugin not found for class: ${node.class}`);
		}
		if (plugin.type !== 'node') {
			throw new Error(`Plugin for class ${node.class} is not a node`);
		}
	} catch (err) {
		console.error('Error getting outputs for node:', nodeId, err);
		return [];
	}
	const outputs = plugin.extractOutputs(node.config);
	return outputs;
}

function getInputs(nodeId: string, node: InstanceDataWithoutType<unknown>): InputInfo[] {
	const plugin = getHttpClassEditorPlugin(node.class);
	try {
		if (!plugin) {
			throw new Error(`Plugin not found for class: ${node.class}`);
		}
		if (plugin.type !== 'node') {
			throw new Error(`Plugin for class ${node.class} is not a node`);
		}
	} catch (err) {
		console.error('Error getting inputs for node:', nodeId, err);
		return [];
	}
	// Currently, we don't have a way to extract inputs from config
	// Assuming default input port for now
	const inputs = plugin.extractInputs(node.config);
	return inputs;
}

export type FlowGraphNode = {
	id: string;
	data: InstanceDataWithoutType<unknown>;
	outputs: OutputInfo[];
	inputs: InputInfo[];
};

export type FlowGraphFilter = {
	id: string;
	data: InstanceDataWithoutType<unknown>;
};

export type FlowGraphLink = {
	from: NodeTargetObject;
	to: NodeTargetObject;
};

export type FlowGraph = {
	entrypoint: string;
	nodes: Record<string, FlowGraphNode>;
	filters: Record<string, FlowGraphFilter>;
	links: {
		indexedByFrom: Map<NodeId, Record<NodePort, NodeTargetObject>>;
		indexedByTo: Map<NodeId, Record<NodePort, NodeTargetObject>>;
	};
	readonly addLink: (from: NodeTargetObject, to: NodeTargetObject) => void;
	readonly addNode: (node: FlowGraphNode) => void;
	readonly deleteLink: (from: NodeTargetObject, to: NodeTargetObject) => void;
	readonly deleteNode: (nodeId: NodeId) => void;
	readonly asTree: () => FlowTreeViewRoot;
};

export const FlowGraph = {
	new(entrypoint: string): FlowGraph {
		const obj = Object.create({});
		obj.entrypoint = entrypoint;
		obj.nodes = {};
		obj.filters = {};
		obj.links = {
			indexedByFrom: new Map(),
			indexedByTo: new Map()
		};
		obj.addLink = (from: NodeTargetObject, to: NodeTargetObject) =>
			FlowGraph.addLink(obj, from, to);
		obj.deleteLink = (from: NodeTargetObject, to: NodeTargetObject) =>
			FlowGraph.deleteLink(obj, from, to);
		obj.addNode = (node: FlowGraphNode) => FlowGraph.addNode(obj, node);
		obj.deleteNode = (nodeId: NodeId) => FlowGraph.deleteNode(obj, nodeId);
		obj.asTree = () => flowGraphAsTree(obj);
		return obj as FlowGraph;
	},
	addNode(self: FlowGraph, node: FlowGraphNode) {
		self.nodes[node.id] = node;
	},
	deleteNode(self: FlowGraph, nodeId: NodeId) {
		delete self.nodes[nodeId];

		// Remove all links from this node
		self.links.indexedByFrom.delete(nodeId);

		// Remove all links to this node
		self.links.indexedByTo.delete(nodeId);

		// Also remove any links in other nodes that point to this node
		for (const [fromNodeId, ports] of self.links.indexedByFrom.entries()) {
			for (const port in ports) {
				if (ports[port].nodeId === nodeId) {
					delete ports[port];
				}
			}
			if (Object.keys(ports).length === 0) {
				self.links.indexedByFrom.delete(fromNodeId);
			}
		}

		for (const [toNodeId, ports] of self.links.indexedByTo.entries()) {
			for (const port in ports) {
				if (ports[port].nodeId === nodeId) {
					delete ports[port];
				}
			}
			if (Object.keys(ports).length === 0) {
				self.links.indexedByTo.delete(toNodeId);
			}
		}
	},
	addLink(self: FlowGraph, from: NodeTargetObject, to: NodeTargetObject) {
		// Add to indexedByFrom
		if (!self.links.indexedByFrom.has(from.nodeId)) {
			self.links.indexedByFrom.set(from.nodeId, {});
		}
		self.links.indexedByFrom.get(from.nodeId)![from.port] = to;

		// Add to indexedByTo
		if (!self.links.indexedByTo.has(to.nodeId)) {
			self.links.indexedByTo.set(to.nodeId, {});
		}
		self.links.indexedByTo.get(to.nodeId)![to.port] = from;
	},

	deleteLink(self: FlowGraph, from: NodeTargetObject, to: NodeTargetObject) {
		// Remove from indexedByFrom
		const fromRecord = self.links.indexedByFrom.get(from.nodeId);
		if (fromRecord && fromRecord[from.port]) {
			delete fromRecord[from.port];
		}

		// Remove from indexedByTo
		const toRecord = self.links.indexedByTo.get(to.nodeId);
		if (toRecord && toRecord[to.port]) {
			delete toRecord[to.port];
		}
	}
};

export async function buildFlowGraph(flow: FlowConfig): Promise<FlowGraph> {
	const resolving: Record<string, Promise<void>> = {};
	const resolvedNodes: Record<string, InstanceDataWithoutType<unknown>> = {};
	// resolve all the config
	for (const [nodeId, nodeData] of Object.entries(flow.nodes ?? {})) {
		if (isLinkValue(nodeData.config)) {
			const promise = api.resolve.link_to_object(nodeData.config).then((response) => {
				resolvedNodes[nodeId] = {
					...nodeData,
					config: response
				};
			});
			resolving[nodeId] = promise;
		} else {
			resolvedNodes[nodeId] = {
				...nodeData
			};
		}
	}
	await Promise.all(Object.values(resolving));
	const graph: FlowGraph = FlowGraph.new(flow.entrypoint);
	for (const [nodeId, nodeData] of Object.entries(resolvedNodes)) {
		const outputs = getOutputs(nodeId, nodeData);
		const inputs = getInputs(nodeId, nodeData);
		const links: FlowGraphLink[] = outputs.map((output) => {
			return {
				from: { nodeId, port: output.port },
				to: output.target
			};
		});

		const node: FlowGraphNode = {
			id: nodeId,
			data: nodeData,
			inputs,
			outputs
		};
		graph.addNode(node);
		for (const link of links) {
			graph.addLink(link.from, link.to);
		}
	}
	for (const [filterId, filterData] of Object.entries(flow.filters ?? {})) {
		const filter: FlowGraphFilter = {
			id: filterId,
			data: filterData
		};
		graph.filters[filterId] = filter;
	}
	return graph;
}

export type FlowTreeViewNode =
	| FlowTreeViewNodeDispatcher
	| FlowTreeViewNodeDispatcherReference
	| FlowTreeViewNodeService
	| FlowTreeViewRoot
	| FlowTreeViewNodeFilter
	| FlowTreeViewNodeFilterDir
	| FlowTreeViewOrphans
	| FlowTreeViewNodeInterface;

export type FlowTreeViewInstanceSelection =
	| {
			type: 'node';
			id: string;
	  }
	| {
			type: 'filter';
			id: string;
	  };

export const FlowTreeViewInstanceSelection = {
	fromString(value: string): FlowTreeViewInstanceSelection | undefined {
		if (value.startsWith('$filter:')) {
			return {
				type: 'filter',
				id: value.substring('$filter:'.length)
			};
		} else if (value.startsWith('$node:')) {
			return {
				type: 'node',
				id: value.substring('$node:'.length)
			};
		}
		return undefined;
	}
};
export const FlowTreeViewNode = {
	nodeToValue(node: FlowTreeViewNode): string {
		switch (node.type) {
			case 'dispatcher':
			case 'service':
				return `$node:${node.id}`;
			case 'dispatcher-reference':
				return `$node:${node.ref}`;
			case 'root':
				return '$root';
			case 'filter':
				return `$filter:${node.id}`;
			case 'filter-dir': {
				let label;
				if (node.location.type === 'global') {
					label = '$global';
				} else {
					label = `${node.location.node}:${node.location.type}:${node.location.port}`;
				}
				return `$filter-dir:${label}`;
			}
			case 'orphans':
				return '$orphans';
			case 'node-interface':
				return `$${node.kind}:${node.node}`;
		}
	},
	nodeToString(node: FlowTreeViewNode): string {
		return FlowTreeViewNode.nodeToValue(node);
	},
	nodeToChildrenCount(node: FlowTreeViewNode): number {
		switch (node.type) {
			case 'dispatcher':
				return node.children.length;
			case 'service':
				return 1;
			case 'dispatcher-reference':
			case 'filter':
				return 0;
			case 'node-interface':
				return node.filters.length;
			case 'root':
				return 3;
			case 'filter-dir':
				return node.filters.length;
			case 'orphans':
				return node.nodes.length;
		}
	},
	nodeToChildren(node: FlowTreeViewNode): FlowTreeViewNode[] {
		switch (node.type) {
			case 'dispatcher':
				return [...node.children, node.outputs, node.inputs];
			case 'filter-dir':
				return node.filters;
			case 'orphans':
				return node.nodes;
			case 'service':
				return [node.inputs];
			case 'dispatcher-reference':
			case 'filter':
				return [];
			case 'root':
				return [node.node_entrypoint, node.orphans, node.filters];
			case 'node-interface':
				return node.filters;
		}
	}
};

export type FlowTreeViewRoot = {
	type: 'root';
	filters: FlowTreeViewNodeFilterDir;
	node_entrypoint: FlowTreeViewNode;
	orphans: FlowTreeViewOrphans;
};

export type FlowTreeViewNodeDispatcherReference = {
	ref: string;
	type: 'dispatcher-reference';
};

export type FlowTreeViewNodeDispatcher = {
	id: string;
	type: 'dispatcher';
	children: FlowTreeViewNode[];
	outputs: FlowTreeViewNodeInterface;
	inputs: FlowTreeViewNodeInterface;
};
export type FlowTreeViewNodeService = {
	id: string;
	inputs: FlowTreeViewNodeInterface;
	type: 'service';
};

export type FlowTreeViewNodeFilter = {
	id: string;
	type: 'filter';
};

export type FlowTreeViewNodeInterface = {
	type: 'node-interface';
	kind: 'input' | 'output';
	node: string;
	filters: FlowTreeViewNodeFilterDir[];
};

export type FlowTreeViewNodeFilterDir = {
	type: 'filter-dir';
	location:
		| {
				type: 'input';
				node: string;
				port: string;
		  }
		| {
				type: 'output';
				port: string;
				node: string;
		  }
		| {
				type: 'global';
		  };
	filters: FlowTreeViewNodeFilter[];
};

export type FlowTreeViewNodeOutputInterface = {
	port: string;
	filters: FlowTreeViewNodeFilter[];
};

export type FlowTreeViewNodeInputInterface = {
	port: string;
	filters: FlowTreeViewNodeFilter[];
};

export type FlowTreeViewOrphans = {
	type: 'orphans';
	nodes: FlowTreeViewNode[];
};

export function flowGraphAsTree(graph: FlowGraph): FlowTreeViewRoot {
	const entrypoint = graph.entrypoint;
	const visited = new Set<string>();
	const orphans: FlowTreeViewNode[] = [];
	console.debug('Building flow tree view from graph with entrypoint:', graph.nodes);
	function buildTreeNode(nodeId: string): FlowTreeViewNode {
		console.debug('Building tree node for:', nodeId);
		// is dispatcher node?
		const node = graph.nodes[nodeId];
		if (node.outputs.length > 0) {
			if (visited.has(nodeId)) {
				return <FlowTreeViewNodeDispatcherReference>{
					ref: nodeId,
					type: 'dispatcher-reference'
				};
			}
			visited.add(nodeId);
			const children: string[] = [];
			const outputs = graph.links.indexedByFrom.get(nodeId);
			if (outputs) {
				for (const target of Object.values(outputs)) {
					children.push(target.nodeId);
				}
			}
			return <FlowTreeViewNodeDispatcher>{
				id: nodeId,
				children: children.map((childId) => buildTreeNode(childId)),
				outputs: <FlowTreeViewNodeInterface>{
					type: 'node-interface',
					kind: 'output',
					node: nodeId,
					filters: node.outputs.map(
						(output) =>
							<FlowTreeViewNodeFilterDir>{
								type: 'filter-dir',
								location: {
									type: 'output',
									port: output.port,
									node: nodeId
								},
								filters: output.filters.map(
									(filterId) =>
										<FlowTreeViewNodeFilter>{
											id: filterId,
											type: 'filter'
										}
								)
							}
					)
				},
				inputs: <FlowTreeViewNodeInterface>{
					type: 'node-interface',
					kind: 'input',
					node: nodeId,
					filters: node.inputs.map(
						(input) =>
							<FlowTreeViewNodeFilterDir>{
								type: 'filter-dir',
								location: {
									type: 'input',
									port: input.port,
									node: nodeId
								},
								filters: input.filters.map(
									(filterId) =>
										<FlowTreeViewNodeFilter>{
											id: filterId,
											type: 'filter'
										}
								)
							}
					)
				},
				type: 'dispatcher'
			};
		} else {
			visited.add(nodeId);
			return <FlowTreeViewNodeService>{
				id: nodeId,
				type: 'service',
				inputs: <FlowTreeViewNodeInterface>{
					type: 'node-interface',
					kind: 'input',
					node: nodeId,
					filters: node.inputs.map(
						(input) =>
							<FlowTreeViewNodeFilterDir>{
								type: 'filter-dir',
								location: {
									type: 'input',
									port: input.port,
									node: nodeId
								},
								filters: input.filters.map(
									(filterId) =>
										<FlowTreeViewNodeFilter>{
											id: filterId,
											type: 'filter'
										}
								)
							}
					)
				}
			};
		}
	}
	const root = buildTreeNode(entrypoint);

	// Find orphans
	for (const nodeId of Object.keys(graph.nodes)) {
		if (!visited.has(nodeId)) {
			orphans.push(buildTreeNode(nodeId));
		}
	}
	const filters: FlowTreeViewNodeFilter[] = [];
	for (const filterId of Object.keys(graph.filters)) {
		filters.push({
			id: filterId,
			type: 'filter'
		});
	}
	return {
		type: 'root',
		node_entrypoint: root,
		filters: {
			type: 'filter-dir',
			location: {
				type: 'global'
			},
			filters
		},
		orphans: {
			type: 'orphans',
			nodes: orphans
		}
	};
}
