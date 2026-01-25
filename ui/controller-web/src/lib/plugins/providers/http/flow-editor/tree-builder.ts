import { getHttpClassEditorPlugin } from '$lib/plugins/registry';

// Types for the Node/Filter data structures
export type NodeData = {
	class?: string;
	config: any;
};

// Tree Node structure for the UI
export type TreeNode = {
	id: string; // The unique ID of the node/filter
	type: 'node' | 'filter';
	data: NodeData;
	children: TreeConnection[];
	isCycle?: boolean; // If this node has already been visited in the current path
	isEntry?: boolean; // If this is the main entry point
};

export type TreeConnection = {
	id: string; // Unique ID for the connection (e.g., nodeID-portName)
	label: string; // The label of the output port
	targetId?: string; // The ID of the target node
	filters: string[]; // List of filter IDs applied to this connection
	targetNode?: TreeNode; // The resolved target node (recursive)
	filterNodes?: TreeNode[]; // The resolved filter nodes
};

// Helper to extract outputs from a node's config using its plugin
function getOutputs(nodeId: string, node: NodeData) {
	if (!node.class) return [];
	
	const plugin = getHttpClassEditorPlugin(node.class, 'node');
	if (!plugin || !plugin.extractOutputs) return [];

	try {
		return plugin.extractOutputs(node.config) || [];
	} catch (e) {
		console.error(`Error extracting outputs for node ${nodeId}:`, e);
		return [];
	}
}

/**
 * Builds a hierarchical tree structure starting from the entrypoint.
 */
export function buildFlowTree(
	nodes: Record<string, NodeData>,
	filters: Record<string, NodeData>,
	entrypoint: string
): { tree: TreeNode | null; orphans: TreeNode[] } {
	
	const visitedInPath = new Set<string>();
	const allVisited = new Set<string>();

	function buildNode(id: string, isEntry: boolean = false): TreeNode {
		const isCycle = visitedInPath.has(id);
		allVisited.add(id);

		const nodeData = nodes[id];
		if (!nodeData) {
            // Handle missing node case gracefully
			return {
				id,
				type: 'node',
				data: { config: {} }, 
				children: [],
                isCycle: false // Or maybe mark as missing?
			};
		}

		if (isCycle) {
			return {
				id,
				type: 'node',
				data: nodeData,
				children: [],
				isCycle: true,
				isEntry
			};
		}

		visitedInPath.add(id);

		const outputs = getOutputs(id, nodeData);
		const children: TreeConnection[] = outputs.map((output) => {
			const connectionId = `${id}-${output.port}`;
			
            // Resolve Filters
			const filterNodes: TreeNode[] = [];
			if (output.filters) {
				output.filters.forEach(filterId => {
					if (filters[filterId]) {
                        // Filters usually don't have children in this context, 
                        // or if they do, we'd need to handle that. 
                        // Assuming filters are just linear processors for now.
						filterNodes.push({
							id: filterId,
							type: 'filter',
							data: filters[filterId],
							children: []
						});
					}
				});
			}

            // Resolve Target Node
			let targetNode: TreeNode | undefined;
			if (output.target) {
				targetNode = buildNode(output.target);
			}

			return {
				id: connectionId,
				label: output.label || output.port,
				targetId: output.target,
				filters: output.filters || [],
				filterNodes,
				targetNode
			};
		});

		visitedInPath.delete(id);

		return {
			id,
			type: 'node',
			data: nodeData,
			children,
			isCycle: false,
			isEntry
		};
	}

	const tree = entrypoint ? buildNode(entrypoint, true) : null;

	// Find orphans (nodes not visited during the tree traversal)
	const orphans: TreeNode[] = [];
	Object.keys(nodes).forEach(nodeId => {
		if (!allVisited.has(nodeId)) {
			orphans.push({
				id: nodeId,
				type: 'node',
				data: nodes[nodeId],
				children: [],
                isCycle: false
			});
		}
	});

	return { tree, orphans };
}

// Unified Tree View Node for Skeleton UI
export type ViewNode = {
	id: string;
	label: string;
	type: 'root' | 'node' | 'filter' | 'connection' | 'orphan-group' | 'missing' | 'cycle';
	data?: any; // Config or specific data
	originalId?: string; // The ID in the nodes/filters map
	children: ViewNode[];
	isEntry?: boolean;
};

/**
 * Converts the logical Flow Tree into a unified View Tree for rendering.
 */
export function toViewTree(
	logicalTree: TreeNode | null,
	orphans: TreeNode[]
): ViewNode {
	const rootChildren: ViewNode[] = [];

	if (logicalTree) {
		rootChildren.push(convertLogicalNode(logicalTree));
	}

	if (orphans.length > 0) {
		rootChildren.push({
			id: 'orphans-group',
			label: 'Disconnected',
			type: 'orphan-group',
			children: orphans.map(node => convertLogicalNode(node)),
			originalId: 'orphans-group'
		});
	}

	return {
		id: 'root',
		label: 'Root',
		type: 'root',
		children: rootChildren,
		originalId: 'root'
	};
}

function convertLogicalNode(node: TreeNode): ViewNode {
	const children: ViewNode[] = [];

	// Process connections (outputs)
	if (node.children) {
		node.children.forEach(conn => {
			const connChildren: ViewNode[] = [];

			// 1. Add Filters
			if (conn.filterNodes) {
				conn.filterNodes.forEach(filter => {
					connChildren.push(convertLogicalNode(filter));
				});
			}

			// 2. Add Target Node
			if (conn.targetNode) {
				connChildren.push(convertLogicalNode(conn.targetNode));
			} else if (conn.targetId) {
				// Missing target
				connChildren.push({
					id: `${conn.id}-missing`,
					label: `Missing: ${conn.targetId}`,
					type: 'missing',
					originalId: conn.targetId,
					children: []
				});
			}

			// Create the Connection (Branch) node
			children.push({
				id: conn.id,
				label: conn.label,
				type: 'connection',
				originalId: conn.id,
				children: connChildren
			});
		});
	}

	return {
		id: node.id, // Use unique ID logic if duplicates exist (cycles/references might need handling)
		label: node.id,
		type: node.type === 'node' && node.isCycle ? 'cycle' : node.type as any,
		data: node.data,
		originalId: node.id,
		children,
		isEntry: node.isEntry
	};
}
