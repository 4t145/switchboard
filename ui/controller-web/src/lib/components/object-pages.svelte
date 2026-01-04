<script lang="ts" module>
    import type { StorageObjectWithoutData, ServiceDescriptor, StorageObjectMeta, StorageObjectDescriptor } from '$lib/api/types';
    import type { ObjectFilter } from '$lib/api/routes/storage';

    import  {Button} from 'bits-ui';
    import KernelStateLabel from './kernel-state-label.svelte';
    import { Info } from 'lucide-svelte';
    interface Props {
        pageSize: number;
        filter: ObjectFilter;
    } 

</script>
<script lang="ts">
	import  { api } from "$lib/api/routes";

    const props: Props = $props();
    const { pageSize, filter } = props;
    let cursor: string | null = null;
    let items = 
    async function loadNextPage() {
        try {
            api.storage.list({
                cursor: {
                    next: cursor
                },
                limit: pageSize
            }, filter);
        } catch (e) {
            console.error("Failed to load storage objects", e);
        }
    }

</script>


<!-- {#await api.sto}
    
{:then } 
    
{/await} -->