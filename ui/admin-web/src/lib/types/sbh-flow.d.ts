export interface Position {
	x: number;
	y: number;
}

export interface Size {
	width: number;
	height: number;
}

export interface NodePort {
	id: string;
	type: 'input' | 'output';
	position: Position;
	connected: boolean;
}

export type NodeType = 'service' | 'router' | 'layer' | 'composition';

export interface FlowNode {
	id: string;
	type: NodeType;
	position: Position;
	size: Size;
	label: string;
	config?: any;
	ports: {
		inputs: NodePort[];
		outputs: NodePort[];
	};
	selected?: boolean;
	dragging?: boolean;
}

export interface Connection {
	id: string;
	sourceNodeId: string;
	sourcePortId: string;
	targetNodeId: string;
	targetPortId: string;
	path?: string; // SVG path for the connection line
}

export interface FlowchartState {
	nodes: FlowNode[];
	connections: Connection[];
	selectedNodes: string[];
	dragState: {
		isDragging: boolean;
		draggedNodeId?: string;
		startPosition?: Position;
		offset?: Position;
	};
	connectionState: {
		isConnecting: boolean;
		sourceNodeId?: string;
		sourcePortId?: string;
		tempLine?: { start: Position; end: Position };
	};
	viewBox: {
		x: number;
		y: number;
		width: number;
		height: number;
		scale: number;
	};
}
