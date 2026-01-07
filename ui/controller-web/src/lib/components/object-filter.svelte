<script lang="ts" module>
    import { Select } from "bits-ui";
    import Check from "phosphor-svelte/lib/Check";
    import Palette from "phosphor-svelte/lib/Palette";
    import CaretUpDown from "phosphor-svelte/lib/CaretUpDown";
    import CaretDoubleUp from "phosphor-svelte/lib/CaretDoubleUp";
    import CaretDoubleDown from "phosphor-svelte/lib/CaretDoubleDown";
    interface Props {
        dataType?: string;
        id?: string;
        revision?: string;
        latestOnly?: boolean;
        createdBefore?: Date;
        createdAfter?: Date;
    } 

</script>
<script lang="ts">
	import  { api } from "$lib/api/routes";
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
        createdBefore = $bindable()
    }: Props = $props();
    const selectedDataTypeLabel = $derived(
        dataType
        ? dataTypeOptions.find((option) => option.value === dataType)?.label
        : "Select a data type"
    );
</script>

<form>
    <label for="id">ID</label>
    <input bind:value={id} type="text" name="id" id="id"/> 
    <label for="revision">Revision</label>
    <input bind:value={revision} type="text" name="revision" id="revision"/>
    <label for="latestOnly">Latest Only</label>
    <input bind:checked={latestOnly} type="checkbox" id="latestOnly"/>
    <Select.Root
        type="single"
        bind:value={dataType}
        items={dataTypeOptions}
        allowDeselect={true}
    >
        <Select.Trigger
        class="h-input rounded-9px border-border-input bg-background data-placeholder:text-foreground-alt/50 inline-flex w-[296px] touch-none select-none items-center border px-[11px] text-sm transition-colors"
        aria-label="Select a data type"
        >
        {selectedDataTypeLabel}
        <CaretUpDown class="text-muted-foreground ml-auto size-6" />
        </Select.Trigger>
        <Select.Portal>
        <Select.Content
            class="focus-override border-muted bg-background shadow-popover data-[state=open]:animate-in data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=open]:fade-in-0 data-[state=closed]:zoom-out-95 data-[state=open]:zoom-in-95 data-[side=bottom]:slide-in-from-top-2 data-[side=left]:slide-in-from-right-2 data-[side=right]:slide-in-from-left-2 data-[side=top]:slide-in-from-bottom-2 outline-hidden z-50 h-96 max-h-[var(--bits-select-content-available-height)] w-[var(--bits-select-anchor-width)] min-w-[var(--bits-select-anchor-width)] select-none rounded-xl border px-1 py-3 data-[side=bottom]:translate-y-1 data-[side=left]:-translate-x-1 data-[side=right]:translate-x-1 data-[side=top]:-translate-y-1"
            sideOffset={10}
        >
            <Select.ScrollUpButton class="flex w-full items-center justify-center">
            <CaretDoubleUp class="size-3" />
            </Select.ScrollUpButton>
            <Select.Viewport class="p-1">
            {#each dataTypeOptions as option, i (i + option.value)}
                <Select.Item
                class="rounded-button data-highlighted:bg-muted outline-hidden data-disabled:opacity-50 flex h-10 w-full select-none items-center py-3 pl-5 pr-1.5 text-sm capitalize"
                value={option.value}
                label={option.label}
                >
                {#snippet children({ selected })}
                    {option.label}
                    {#if selected}
                    <div class="ml-auto">
                        <Check aria-label="check" />
                    </div>
                    {/if}
                {/snippet}
                </Select.Item>
            {/each}
            </Select.Viewport>
            <Select.ScrollDownButton class="flex w-full items-center justify-center">
            <CaretDoubleDown class="size-3" />
            </Select.ScrollDownButton>
        </Select.Content>
        </Select.Portal>
    </Select.Root>

</form>