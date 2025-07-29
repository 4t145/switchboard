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
	viewBox: ViewBoxState;
}
export interface ViewBoxState {
	position: Position;
	width: number;
	height: number;
	scale: number;
}
export class ViewBox implements ViewBoxState {
	public position: Position;
	public scale: number;
	public canvas: SVGSVGElement;
	public width: number;
	public height: number;
	constructor(canvas: SVGSVGElement, state: ViewBoxState) {
		this.position = state.position;
		this.scale = state.scale;
		this.width = state.width;
		this.height = state.height;
		this.canvas = canvas;
	}
	actualScale(): number {
		const rect = this.canvas.getBoundingClientRect();
		const svgWidth = rect.width;
		const svgHeight = rect.height;

		// 计算实际的缩放比例
		const scaleX = svgWidth / this.width;
		const scaleY = svgHeight / this.height;
		const actualScale = Math.min(scaleX, scaleY);
		return actualScale
	}
	canvasCoordToClientCoord(
		canvasCoord: Position,
	): Position {
		const rect = this.canvas.getBoundingClientRect();
		const scale = this.actualScale();
		const x = (canvasCoord.x - this.position.x) * scale + rect.left;
		const y = (canvasCoord.y - this.position.y) * scale + rect.top;
		return { x, y };
	}
	clientCoordToCanvasCoord(
		clientCoord: Position,
	): Position {
		const rect = this.canvas.getBoundingClientRect();
		const scale = this.actualScale();
		const x = (clientCoord.x - rect.left) / scale + this.position.x;
		const y = (clientCoord.y - rect.top) / scale + this.position.y;
		return { x, y };
	}
	canvasCenterCoord(): Position {
		const x = this.position.x + this.width / 2;
		const y = this.position.y + this.height / 2;
		return { x, y };
	}
}