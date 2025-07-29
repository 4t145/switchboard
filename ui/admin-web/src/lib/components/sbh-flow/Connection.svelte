<script lang="ts">
    import type { Connection, FlowNode } from '$lib/types/sbh-flow';

    interface Props {
        connection: Connection;
        nodes: FlowNode[];
        selected?: boolean;
    }

    let { connection, nodes, selected = false }: Props = $props();

    // Find source and target positions
    const sourceNode = $derived(nodes.find(n => n.id === connection.sourceNodeId));
    const targetNode = $derived(nodes.find(n => n.id === connection.targetNodeId));
    
    const sourcePort = $derived(
        sourceNode?.ports.outputs.find(p => p.id === connection.sourcePortId)
    );
    const targetPort = $derived(
        targetNode?.ports.inputs.find(p => p.id === connection.targetPortId)
    );

    const path = $derived(() => {
        if (!sourceNode || !targetNode || !sourcePort || !targetPort) {
            return '';
        }

        const start = {
            x: sourceNode.position.x + sourcePort.position.x,
            y: sourceNode.position.y + sourcePort.position.y
        };
        
        const end = {
            x: targetNode.position.x + targetPort.position.x,
            y: targetNode.position.y + targetPort.position.y
        };

        // Create a smooth curved path
        const dx = end.x - start.x;
        const dy = end.y - start.y;
        const controlPoint1X = start.x + dx * 0.5;
        const controlPoint2X = end.x - dx * 0.5;

        return `M ${start.x} ${start.y} C ${controlPoint1X} ${start.y} ${controlPoint2X} ${end.y} ${end.x} ${end.y}`;
    });
</script>

{#if path}
    <path
        d={path()}
        class="fill-none stroke-2 {selected ? 'stroke-blue-500' : 'stroke-gray-400'} hover:stroke-blue-400 cursor-pointer"
        stroke-width="2"
        marker-end="url(#arrowhead)"
    />
{/if}