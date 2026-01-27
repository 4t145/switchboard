import dagre from '@dagrejs/dagre';
import type { FlowGraph } from './flow-view-builder';
import type { Edge, Node } from '@xyflow/svelte';

export type FlowGraphLayoutData = {
	nodes: Node[];
	edges: Edge[];
};

const NODE_WIDTH = 200;
const NODE_HEIGHT = 120;

export function convertFlowGraphToSvelteFlow(graph: FlowGraph): FlowGraphLayoutData {
	const dagreGraph = new dagre.graphlib.Graph();
	dagreGraph.setDefaultEdgeLabel(() => ({}));
	dagreGraph.setGraph({
		rankdir: 'LR',
		nodesep: 100,
		ranksep: 100
	});

	const nodeIds = Object.keys(graph.nodes);

	nodeIds.forEach((nodeId) => {
		dagreGraph.setNode(nodeId, {
			width: NODE_WIDTH,
			height: NODE_HEIGHT
		});
	});

	graph.links.indexedByFrom.forEach((targets, fromNodeId) => {
		Object.entries(targets).forEach(([fromPort, target]) => {
			dagreGraph.setEdge(fromNodeId, target.nodeId);
		});
	});

	dagre.layout(dagreGraph);

	const nodes: Node[] = nodeIds.map((nodeId) => {
		const graphNode = graph.nodes[nodeId];
		const nodeWithPosition = dagreGraph.node(nodeId);

		return {
			id: nodeId,
			position: {
				x: nodeWithPosition.x - NODE_WIDTH / 2,
				y: nodeWithPosition.y - NODE_HEIGHT / 2
			},
			data: {
				id: nodeId,
				type: graphNode.data.class,
				inputs: graphNode.inputs,
				outputs: graphNode.outputs,
				isEntrypoint: nodeId === graph.entrypoint,
				config: graphNode.data.config
			},
			type: 'custom'
		};
	});

	const edges: Edge[] = [];
	graph.links.indexedByFrom.forEach((targets, fromNodeId) => {
		Object.entries(targets).forEach(([fromPort, target]) => {
			const edge: Edge = {
				id: `${fromNodeId}-${fromPort}-${target.nodeId}-${target.port}`,
				source: fromNodeId,
				target: target.nodeId,
				sourceHandle: fromPort,
				targetHandle: target.port,
				type: 'smoothstep',
				animated: false
			};
			edges.push(edge);
		});
	});

	return {
		nodes,
		edges
	};
}
