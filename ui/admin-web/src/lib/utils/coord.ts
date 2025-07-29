import type { Position } from "$lib/types/sbh-flow";

export function screenCoordToCanvasCoord(
    screenCoord: Position,
    canvas: HTMLElement,
    zoom: number = 1.0,
): Position {
    const rect = canvas.getBoundingClientRect();
    const x = (screenCoord.x - rect.left) / zoom;
    const y = (screenCoord.y - rect.top) / zoom;
    return { x, y };
}

export function canvasCoordToScreenCoord(
    canvasCoord: Position,
    canvas: HTMLElement,
    zoom: number = 1.0,
): Position {
    const rect = canvas.getBoundingClientRect();
    const x = canvasCoord.x * zoom + rect.left;
    const y = canvasCoord.y * zoom + rect.top;
    return { x, y };
}


export class ViewBox {
    public position: Position;
    public scale: number;
    public canvas: HTMLElement;
    constructor(canvas: HTMLElement, position: Position = { x: 0, y: 0 }, scale: number = 1.0) {
        this.position = position;
        this.scale = scale;
        this.canvas = canvas;
    }
    canvasCoordToClientCoord(
        canvasCoord: Position,
    ): Position {
        const rect = this.canvas.getBoundingClientRect();
        const x = (canvasCoord.x - this.position.x) * this.scale + rect.left;
        const y = (canvasCoord.y - this.position.y) * this.scale + rect.top;
        return { x, y };
    }
    clientCoordToCanvasCoord(
        clientCoord: Position,
    ): Position {
        const rect = this.canvas.getBoundingClientRect();
        const x = (clientCoord.x - rect.left) / this.scale + this.position.x;
        const y = (clientCoord.y - rect.top) / this.scale + this.position.y;
        return { x, y };
    }
}