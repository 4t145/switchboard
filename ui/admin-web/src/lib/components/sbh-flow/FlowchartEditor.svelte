<script module>
	import type { BuiltInNode, BuiltInEdge } from '@xyflow/svelte';

	// Custom nodes
	import type { LayerNodeType } from './nodes/Layer.svelte';
	import type { DanglingNodeType } from './nodes/Dangling.svelte';
	import type { ServiceNodeType } from './nodes/Service.svelte';
	import type { NoteNodeType } from './nodes/Note.svelte';
	import type { RouterNodeType } from './nodes/Router.svelte';
	import type { IncomeNodeType } from './nodes/Income.svelte';

	export type NodeType =
		| BuiltInNode
		| LayerNodeType
		| DanglingNodeType
		| ServiceNodeType
		| NoteNodeType
		| RouterNodeType
		| IncomeNodeType;
	export type EdgeType = BuiltInEdge;
</script>

<script lang="ts">
	import { Background, BackgroundVariant, Controls, MiniMap, SvelteFlow } from '@xyflow/svelte';
	import '@xyflow/svelte/dist/style.css';
	import Layer from './nodes/Layer.svelte';
	import Dangling from './nodes/Dangling.svelte';
	import Service from './nodes/Service.svelte';
	import Note from './nodes/Note.svelte';
	import Router from './nodes/Router.svelte';
	import Income from './nodes/Income.svelte';
	const nodeTypes = {
		layer: Layer,
		dangling: Dangling,
		service: Service,
		note: Note,
		income: Income,
		router: Router
	};
	let nodes = $state.raw(<NodeType[]>[
		{
			id: '$income',
			position: { x: 0, y: -100 },
			data: { class: 'class1', name: 'Layer 1', connection: { source: null, target: '2' } },
			type: 'income'
		},
		{
			id: '1',
			position: { x: 0, y: 0 },
			data: { class: 'class1', name: 'Layer 1', connection: { source: null, target: '2' } },
			type: 'layer'
		},
		{
			id: '2',
			position: { x: 0, y: 100 },
			data: { class: 'class2', name: 'Layer 2', connection: { source: '1', target: null } },
			type: 'layer'
		},
		{
			id: '3',
			position: { x: 0, y: 200 },
			data: { connection: { source: '2' } },
			type: 'dangling'
		},
		{
			id: '4',
			position: { x: 0, y: 300 },
			data: { class: 'service1', name: 'Service 1', connection: { source: null, target: null } },
			type: 'service'
		},
		{
			id: '5',
			position: { x: 0, y: 400 },
			data: { content: 'This is a note' },
			type: 'note'
		},
		{
			id: '6',
			position: { x: 0, y: 500 },
			data: { class: 'router1', name: 'Router 1', connection: { output: {} } },
			type: 'router'
		}
	]);

	let edges = $state.raw(<BuiltInEdge[]>[
		{ id: 'e1-2', source: '1', target: '2' },
		{ id: 'e2-3', source: '2', target: '3' }
	]);
</script>

<SvelteFlow bind:nodes bind:edges {nodeTypes} fitView>
	<Controls position="bottom-left" showFitView showLock showZoom />
	<Background variant={BackgroundVariant.Dots} />
	<MiniMap nodeColor= { node => {
		if (node.type === 'income') return '#000000'; // black
		if (node.type === 'layer') return '#000000'; // black
		if (node.type === 'dangling') return 'orange'; // yellow
		if (node.type === 'service') return '#ec4899'; // pink-500
		if (node.type === 'note') return '#fef08a'; // yellow-50
		if (node.type === 'router') return '#22d3ee'; // cyan-500
		return '#9ca3af'; // gray for others
	}}/>
</SvelteFlow>
