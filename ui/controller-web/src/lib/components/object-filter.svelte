<script lang="ts" module>
    import { Select } from "bits-ui";
    import Check from "phosphor-svelte/lib/Check";
    import Palette from "phosphor-svelte/lib/Palette";
    import CaretUpDown from "phosphor-svelte/lib/CaretUpDown";
    import CaretDoubleUp from "phosphor-svelte/lib/CaretDoubleUp";
    import CaretDoubleDown from "phosphor-svelte/lib/CaretDoubleDown";
    import { Collapsible } from '@skeletonlabs/skeleton-svelte';
    interface Props {
        dataType?: string;
        id?: string;
        revision?: string;
        latestOnly?: boolean;
        createdBefore?: Date;
        createdAfter?: Date;
        onSubmit?: (filter: ObjectFilter) => void;
    } 

</script>
<script lang="ts">
	import  { api } from "$lib/api/routes";
	import type { ObjectFilter } from "$lib/api/routes/storage";
	import { ArrowDownIcon, ArrowUpDownIcon, BrushCleaningIcon, SearchIcon } from "lucide-svelte";
    const dataTypeOptions = [
        { label: "Service Config", value: "service_config" },
        { label: "Pem", value: "pem" },
    ];
    let {
        dataType = $bindable(),
        id = $bindable(),
        revision = $bindable(),
        latestOnly = $bindable(),
        createdAfter = $bindable(),
        createdBefore = $bindable(),
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
        dataType = '';
        id = '';
        revision = '';
        latestOnly = false;
        createdAfter = undefined;
        createdBefore = undefined;
    }
</script>

<form class="w-full space-y-4 p-4">
    <fieldset class="grid gap-4 md:grid-cols-3">
        <legend>Basic Filters</legend>
        <label for="id" class="label">
            <span class="label-text">ID</span>
            <input bind:value={id} type="text" name="id" id="id" class="input w-full"/>
        </label>
        <label for="revision" class="label">
            <span class="label-text">Revision</span>
            <input bind:value={revision} type="text" name="revision" id="revision" class="input w-full"/>
        </label>
        <label for="date-type" class="label">
            <span class="label-text">Data Type</span>
            <select class="select w-full" bind:value={dataType} id="date-type">
                <option value="">All</option>
                {#each dataTypeOptions as option}
                    <option value={option.value}>{option.label}</option>
                {/each}
            </select>
        </label>
    </fieldset>
    <fieldset class="grid gap-4 md:grid-cols-2">
        <legend>
            Advanced Filters
            <button class = "btn-icon hover:preset-tonal">
                {#if advancedOpen}
                    <CaretDoubleUp size={16} onclick={() => (advancedOpen = false)} />
                {:else}
                    <CaretDoubleDown size={16} onclick={() => (advancedOpen = true)} />
                {/if}

            </button>
        </legend>
        <label for="latestOnly" class={`label ${advancedOpen ? '' : 'hidden'}`}>
            <span class="label-text">Latest Only</span>
            <input bind:checked={latestOnly} type="checkbox" id="latestOnly" class="checkbox"/>
        </label>
    </fieldset>
    <fieldset class="flex flex-wrap gap-3 justify-end mt-4">
        <button
                onclick={submit}
                type="button"
                class="btn preset-outlined-primary"
            >
            <SearchIcon size={18} />
            <span>
                搜索
            </span>
        </button>
        <button 
            onclick={resetFilters}
            type="button"
            class="btn preset-outlined"
        >
            <BrushCleaningIcon size={18} />
            <span>
                重置
            </span>
        </button>
    </fieldset>
</form>