<script lang="ts">
    import { createEventDispatcher } from 'svelte';
    import { Button } from 'bits-ui';
    import { Trash2, Copy, Edit } from 'lucide-svelte';

    interface Props {
        selectedNodes: string[];
    }

    let { selectedNodes }: Props = $props();

    const dispatch = createEventDispatcher<{
        delete: void;
        copy: void;
        edit: void;
    }>();
</script>

<div class="absolute top-4 right-4 flex items-center space-x-2 bg-white dark:bg-gray-800 rounded-lg shadow-lg border border-gray-200 dark:border-gray-700 p-2">
    <span class="text-sm text-gray-600 dark:text-gray-400">
        {selectedNodes.length} 个节点已选中
    </span>
    
    <div class="w-px h-6 bg-gray-200 dark:bg-gray-600"></div>
    
    <Button.Root
        class="p-2 text-gray-600 hover:text-blue-600 hover:bg-blue-50 dark:text-gray-400 dark:hover:text-blue-400 dark:hover:bg-blue-900/20 rounded"
        onclick={() => dispatch('edit')}
        disabled={selectedNodes.length !== 1}
    >
        <Edit class="w-4 h-4" />
    </Button.Root>
    
    <Button.Root
        class="p-2 text-gray-600 hover:text-green-600 hover:bg-green-50 dark:text-gray-400 dark:hover:text-green-400 dark:hover:bg-green-900/20 rounded"
        onclick={() => dispatch('copy')}
    >
        <Copy class="w-4 h-4" />
    </Button.Root>
    
    <Button.Root
        class="p-2 text-gray-600 hover:text-red-600 hover:bg-red-50 dark:text-gray-400 dark:hover:text-red-400 dark:hover:bg-red-900/20 rounded"
        onclick={() => dispatch('delete')}
    >
        <Trash2 class="w-4 h-4" />
    </Button.Root>
</div>