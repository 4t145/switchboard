<script lang="ts">
    import { createEventDispatcher } from 'svelte';
    import { Button } from 'bits-ui';
    import { 
        Activity, 
        Route, 
        Layers, 
        Package,
        Circle,
        Settings
    } from 'lucide-svelte';
    import type { FlowNode, NodePort, Position } from '$lib/types/sbh-flow';

    interface Props {
        node: FlowNode;
        scale: number;
    }

    let { node, scale }: Props = $props();

    const dispatch = createEventDispatcher<{
        nodeSelect: { nodeId: string; multiSelect: boolean };
        nodeDragStart: { nodeId: string; position: Position };
        nodeDrag: { nodeId: string; position: Position };
        nodeDragEnd: { nodeId: string };
        portMouseDown: { nodeId: string; portId: string; portType: 'input' | 'output' };
        portMouseUp: { nodeId: string; portId: string; portType: 'input' | 'output' };
        nodeDoubleClick: { nodeId: string };
    }>();

    const nodeTypeConfig = {
        service: {
            icon: Activity,
            color: 'bg-blue-500',
            borderColor: 'border-blue-600',
            textColor: 'text-blue-50'
        },
        router: {
            icon: Route,
            color: 'bg-green-500',
            borderColor: 'border-green-600',
            textColor: 'text-green-50'
        },
        layer: {
            icon: Layers,
            color: 'bg-purple-500',
            borderColor: 'border-purple-600',
            textColor: 'text-purple-50'
        },
        composition: {
            icon: Package,
            color: 'bg-orange-500',
            borderColor: 'border-orange-600',
            textColor: 'text-orange-50'
        }
    };

    const config = nodeTypeConfig[node.type];
    const IconComponent = config.icon;

    let dragStartPosition: Position | null = null;
    let isDragging = false;

    function handleMouseDown(event: MouseEvent) {
        if (event.button !== 0) return; // Only left click
        
        event.preventDefault();
        event.stopPropagation();
        
        dragStartPosition = { x: event.clientX, y: event.clientY };
        isDragging = false;

        dispatch('nodeSelect', { 
            nodeId: node.id, 
            multiSelect: event.ctrlKey || event.metaKey 
        });

        function handleMouseMove(event: MouseEvent) {
            if (!dragStartPosition) return;
            
            const deltaX = event.clientX - dragStartPosition.x;
            const deltaY = event.clientY - dragStartPosition.y;
            
            if (!isDragging && (Math.abs(deltaX) > 3 || Math.abs(deltaY) > 3)) {
                isDragging = true;
                dispatch('nodeDragStart', { nodeId: node.id, position: node.position });
            }
            
            if (isDragging) {
                dispatch('nodeDrag', { 
                    nodeId: node.id, 
                    position: {
                        x: node.position.x + deltaX / scale,
                        y: node.position.y + deltaY / scale
                    }
                });
            }
        }

        function handleMouseUp() {
            if (isDragging) {
                dispatch('nodeDragEnd', { nodeId: node.id });
            }
            
            dragStartPosition = null;
            isDragging = false;
            document.removeEventListener('mousemove', handleMouseMove);
            document.removeEventListener('mouseup', handleMouseUp);
        }

        document.addEventListener('mousemove', handleMouseMove);
        document.addEventListener('mouseup', handleMouseUp);
    }

    function handlePortMouseDown(port: NodePort, event: MouseEvent) {
        event.stopPropagation();
        dispatch('portMouseDown', { 
            nodeId: node.id, 
            portId: port.id, 
            portType: port.type 
        });
    }

    function handlePortMouseUp(port: NodePort, event: MouseEvent) {
        event.stopPropagation();
        dispatch('portMouseUp', { 
            nodeId: node.id, 
            portId: port.id, 
            portType: port.type 
        });
    }

    function handleDoubleClick(event: MouseEvent) {
        event.preventDefault();
        event.stopPropagation();
        dispatch('nodeDoubleClick', { nodeId: node.id });
    }
</script>

<g
    transform="translate({node.position.x}, {node.position.y})"
    class="flow-node {node.selected ? 'selected' : ''}"
>
    <!-- Node body -->
    <rect
        x="0"
        y="0"
        width={node.size.width}
        height={node.size.height}
        rx="8"
        class="fill-white stroke-2 {config.borderColor} {node.selected ? 'stroke-blue-400' : ''}"
        style="filter: drop-shadow(0 4px 6px rgba(0, 0, 0, 0.1))"
    />
    
    <!-- Header -->
    <rect
        x="0"
        y="0"
        width={node.size.width}
        height="32"
        rx="8"
        class="fill-current {config.color}"
    />
    <rect
        x="0"
        y="24"
        width={node.size.width}
        height="8"
        class="fill-current {config.color}"
    />
    
    <!-- Icon -->
    <foreignObject x="8" y="6" width="20" height="20">
        <IconComponent class="w-5 h-5 {config.textColor}" />
    </foreignObject>
    
    <!-- Title -->
    <text
        x="36"
        y="20"
        class="fill-current {config.textColor} text-sm font-medium"
        dominant-baseline="middle"
    >
        {node.type.toUpperCase()}
    </text>
    
    <!-- Label -->
    <text
        x="12"
        y="52"
        class="fill-gray-800 text-sm"
        dominant-baseline="middle"
    >
        {node.label}
    </text>
    
    <!-- Config button -->
    <foreignObject x={node.size.width - 28} y="40" width="20" height="20">
        <Button.Root
            class="w-5 h-5 p-0 bg-transparent hover:bg-gray-100 rounded"
            onclick={() => dispatch('nodeDoubleClick', { nodeId: node.id })}
        >
            <Settings class="w-3 h-3 text-gray-500" />
        </Button.Root>
    </foreignObject>
    
    <!-- Input ports -->
    {#each node.ports.inputs as port}
        <circle
            cx={port.position.x}
            cy={port.position.y}
            r="6"
            class="fill-gray-300 stroke-2 stroke-gray-500 hover:fill-blue-300 cursor-pointer"
            onmousedown={(e) => handlePortMouseDown(port, e)}
            onmouseup={(e) => handlePortMouseUp(port, e)}
        />
    {/each}
    
    <!-- Output ports -->
    {#each node.ports.outputs as port}
        <circle
            cx={port.position.x}
            cy={port.position.y}
            r="6"
            class="fill-gray-300 stroke-2 stroke-gray-500 hover:fill-green-300 cursor-pointer"
            onmousedown={(e) => handlePortMouseDown(port, e)}
            onmouseup={(e) => handlePortMouseUp(port, e)}
        />
    {/each}
    
    <!-- Invisible interaction area -->
    <rect
        x="0"
        y="0"
        width={node.size.width}
        height={node.size.height}
        class="fill-transparent cursor-move"
        onmousedown={handleMouseDown}
        ondblclick={handleDoubleClick}
    />
</g>

<style>
    .flow-node.selected rect:first-child {
        stroke-width: 3;
    }
</style>