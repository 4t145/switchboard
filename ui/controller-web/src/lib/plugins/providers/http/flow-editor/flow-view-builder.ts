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
	if (!plugin) {
		throw new Error(`Plugin not found for class: ${node.class}`);
	}
	if (plugin.type !== 'node') {
		throw new Error(`Plugin for class ${node.class} is not a node`);
	}
	const outputs = plugin.extractOutputs(node.config);
	return outputs;
}

function getInputs(nodeId: string, node: InstanceDataWithoutType<unknown>): InputInfo[] {
	const plugin = getHttpClassEditorPlugin(node.class);
	if (!plugin) {
		throw new Error(`Plugin not found for class: ${node.class}`);
	}
	if (plugin.type !== 'node') {
		throw new Error(`Plugin for class ${node.class} is not a node`);
	}
	// Currently, we don't have a way to extract inputs from config
	// Assuming default input port for now
	const inputs = plugin.extractInputs(node.config);
	return inputs;
}

export type FlowGraphNode = {
	id: string;
	data: InstanceDataWithoutType<unknown>;
	outputs: string[];
	inputs: string[];
};

export type FlowGraphLink = {
	from: NodeTargetObject;
	to: NodeTargetObject;
};

export type FlowGraph = {
	entrypoint: string;
	nodes: Record<string, FlowGraphNode>;
	links: {
		indexedByFrom: Map<NodeId, Record<NodePort, NodeTargetObject>>;
		indexedByTo: Map<NodeId, Record<NodePort, NodeTargetObject>>;
	};
	readonly addLink: (from: NodeTargetObject, to: NodeTargetObject) => void;
	readonly addNode: (node: FlowGraphNode) => void;
	readonly deleteLink: (from: NodeTargetObject, to: NodeTargetObject) => void;
	readonly deleteNode: (nodeId: NodeId) => void;
	readonly asTree: () => FlowTreeView;
};

export const FlowGraph = {
	new(entrypoint: string): FlowGraph {
		const obj = Object.create({});
		obj.entrypoint = entrypoint;
		obj.nodes = {};
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
	const nodes = flow.nodes ?? {};
	const resolving: Record<string, Promise<void>> = {};
	// resolve all the config
	for (const [nodeId, nodeData] of Object.entries(nodes)) {
		if (isLinkValue(nodeData.config)) {
			const promise = api.resolve.link_to_object(nodeData.config).then((response) => {
				nodeData.config = response;
			});
			resolving[nodeId] = promise;
		}
	}
	await Promise.all(Object.values(resolving));
	const graph: FlowGraph = FlowGraph.new(flow.entrypoint);
	for (const [nodeId, nodeData] of Object.entries(nodes)) {
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
			inputs: inputs.map((input) => input.port),
			outputs: outputs.map((output) => output.port)
		};
		graph.addNode(node);
		for (const link of links) {
			graph.addLink(link.from, link.to);
		}
	}
	return graph;
}

export type FlowTreeViewNode = {
	id: string;
	children: FlowTreeViewNode[];
	duplicate: boolean;
	graphReference: FlowGraph;
};

export type FlowTreeView = {
	root: FlowTreeViewNode;
	orphans: FlowTreeViewNode[];
};

export function flowGraphAsTree(graph: FlowGraph): FlowTreeView {
	const entrypoint = graph.entrypoint;
	const visited = new Set<string>();
	const orphans: FlowTreeViewNode[] = [];
	function buildTreeNode(nodeId: string): FlowTreeViewNode {
		if (visited.has(nodeId)) {
			return {
				id: nodeId,
				children: [],
				duplicate: true,
				graphReference: graph
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
		return {
			id: nodeId,
			children: children.map((childId) => buildTreeNode(childId)),
			duplicate: false,
			graphReference: graph
		};
	}
	const root = buildTreeNode(entrypoint);

	// Find orphans
	for (const nodeId of Object.keys(graph.nodes)) {
		if (!visited.has(nodeId)) {
			orphans.push(buildTreeNode(nodeId));
		}
	}

	return {
		root,
		orphans
	};
}
