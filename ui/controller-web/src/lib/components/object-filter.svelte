<script lang="ts" module>
    interface Props {
        dataType?: string;
        id?: string;
        revision?: string;
        latestOnly?: boolean;
        createdBefore?: Date;
        createdAfter?: Date;
        lockedFields?: string[];
        compact?: boolean;
        onSubmit?: (filter: ObjectFilter) => void;
    } 

</script>
<script lang="ts">
	import  { api } from "$lib/api/routes";
	import type { ObjectFilter } from "$lib/api/routes/storage";
	import { ArrowDownIcon, ArrowUpDownIcon, BrushCleaningIcon, SearchIcon } from "lucide-svelte";
    const dataTypeOptions = [
        { label: "Service Config", value: "ServiceConfig" },
        { label: "Pem", value: "pem" },
    ];
    let {
        dataType = $bindable(),
        id = $bindable(),
        revision = $bindable(),
        latestOnly = $bindable(true),
        createdAfter = $bindable(),
        createdBefore = $bindable(),
        lockedFields = [],
        compact = false,
        onSubmit = (filter: ObjectFilter) => {}
    }: Props = $props();
    const selectedDataTypeLabel = $derived(
        dataType
        ? dataTypeOptions.find((option) => option.value === dataType)?.label
        : "Select a data type"
    );
    let advancedOpen = $state(false);
    function submit() {
        onSubmit({
            ...(dataType ? { data_type: dataType } : {}),
            ...(id ? { id } : {}),
            ...(revision ? { revision } : {}),
            ...(latestOnly ? { latest_only: latestOnly } : {}),
            ...(createdAfter ? { created_after: createdAfter } : {}),
            ...(createdBefore ? { created_before: createdBefore } : {}),
        });
    }
    function resetFilters() {
        if (!lockedFields.includes('dataType')) dataType = '';
        if (!lockedFields.includes('id')) id = '';
        if (!lockedFields.includes('revision')) revision = '';
        if (!lockedFields.includes('latestOnly')) latestOnly = true;
        createdAfter = undefined;
        createdBefore = undefined;
    }
</script>

<form class="flex flex-wrap gap-4 items-center p-2 w-full">
    <!-- Compact View -->
    {#if !lockedFields.includes('id')}
        <label class="flex items-center gap-2">
            <span class="text-sm font-medium opacity-75">ID:</span>
            <input bind:value={id} type="text" placeholder="Filter by ID..." class="input input-sm w-48"/>
        </label>
    {/if}
    
    {#if !lockedFields.includes('revision')}
        <label class="flex items-center gap-2">
            <span class="text-sm font-medium opacity-75">Rev:</span>
            <input bind:value={revision} type="text" placeholder="SHA..." class="input input-sm w-32"/>
        </label>
    {/if}

    {#if !lockedFields.includes('dataType')}
        <label class="flex items-center gap-2">
            <span class="text-sm font-medium opacity-75">Type:</span>
                <select class="select select-sm w-40" bind:value={dataType}>
                <option value="">All Types</option>
                {#each dataTypeOptions as option}
                    <option value={option.value}>{option.label}</option>
                {/each}
            </select>
        </label>
    {/if}
    
    {#if !lockedFields.includes('latestOnly')}
        <label class="flex items-center gap-2 cursor-pointer">
            <input bind:checked={latestOnly} type="checkbox" class="checkbox checkbox-sm"/>
            <span class="text-sm font-medium">Latest Only</span>
        </label>
    {/if}

    <div class="flex gap-2 ml-auto">
        <button onclick={submit} type="button" class="btn btn-sm preset-filled-primary" title="Search">
            <SearchIcon size={16} class="mr-1" /> Search
        </button>
        <button onclick={resetFilters} type="button" class="btn-icon btn-icon-sm preset-outlined" title="Reset">
            <BrushCleaningIcon size={16} />
        </button>
    </div>
</form>