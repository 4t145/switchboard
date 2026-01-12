<script lang="ts" module>

    import CaretDoubleUp from "phosphor-svelte/lib/CaretDoubleUp";
    import CaretDoubleDown from "phosphor-svelte/lib/CaretDoubleDown";
    interface Props {
        resolver: string;
        config?: Record<string, any> | null;
        saveAs?: string | null;
        onSubmit: (buildForm: ResolveServiceConfigRequest) => void;
    } 

</script>
<script lang="ts">
	import  { api } from "$lib/api/routes";
	import type { ObjectFilter } from "$lib/api/routes/storage";
	import { ArrowDownIcon, ArrowUpDownIcon, BrushCleaningIcon, FolderTreeIcon, NetworkIcon, SearchIcon, WrenchIcon } from "lucide-svelte";
	import type { ResolveServiceConfigRequest } from "$lib/api/routes/resolve";
    const dataTypeOptions = [
        { label: "Service Config", value: "ServiceConfig" },
        { label: "Pem", value: "pem" },
    ];
    let {
        resolver = $bindable("fs"),
        saveAs = $bindable(null as string | null),
        config = $bindable(),
        onSubmit = (request: ResolveServiceConfigRequest) => {}
    }: Props = $props();
    const icons = {
        fs: FolderTreeIcon,
        k8s: NetworkIcon,
    }
    const resolverOptions = <{
        label: string;
        value: string;
        icon: keyof typeof icons;
    }[]>[
        { label: "Filesystem", value: "fs", icon: "fs" },
        { label: "Kubernetes", value: "k8s", icon: "k8s" },
    ]
    let advancedOpen = $state(false);
    function clearSaveAs() {
        saveAs = null;
    }
    function clearFsConfigFieldPath() {
        fsConfigFieldPath = null;
    }
    function submit() {
        onSubmit({
            resolver,
            config,
            save_as: saveAs ?? undefined,
        });
    }
    let fsConfigFieldPath = $state(null as string | null);

    $effect(
        () => {
            if (resolver === 'fs') {
                config = {
                    path: fsConfigFieldPath
                }
            }
        }
    )
</script>

<form class="w-full space-y-4 p-4 flex flex-col">
    <fieldset class="grid gap-4 md:grid-cols-2">
        <label for="resolver" class="label">
            <span class="label-text">Resolver</span>
            <select class="select w-full" bind:value={resolver} id="resolver" name="resolver" required>
                {#each resolverOptions as option}
                    <option value={option.value}> {option.label}</option>
                {/each}
            </select>
        </label>
        <label for="save-as" class="label">
            <span class="label-text">Save As</span>
            <div class="input-group grid-cols-[1fr_auto]">
                <input bind:value={saveAs} type="text" name="save-as" id="save-as" class="ig-input w-full"  placeholder="Don't save"
                onchange={(e) => {
                    if (saveAs?.length === 0) {
                        saveAs = null;
                    }
                }}/>
                <button 
                    type="button" 
                    onclick={clearSaveAs}
                    class="ig-btn preset-outlined-surface-500 hover:preset-filled-error-500"
                    title="清除"
                >
                ✕
                </button>
            </div>
        </label>
    </fieldset>
    {#if resolver!== null  && resolver !== undefined && resolver.length > 0}
    <fieldset class="grid gap-4 md:grid-cols-2">
        <legend class="config">Config of 
            <span class="badge preset-filled mx-1">
                {resolverOptions.find(option => option.value === resolver)?.label}
            </span>
        </legend>
        {#if resolver === 'fs'}
            <label for="config-path" class="label col-span-2">
                <span class="label-text">Config Path</span>
                <div class="input-group grid-cols-[1fr_auto]">
                    <input 
                        bind:value={fsConfigFieldPath} 
                        type="text" 
                        name="config-path" 
                        id="config-path" 
                        class="ig-input w-full"
                        placeholder="Use default path"
                    />
                    <button 
                        type="button" 
                        onclick={clearFsConfigFieldPath}
                        class="ig-btn preset-outlined-surface-500 hover:preset-filled-error-500"
                        title="清除"
                    >
                    ✕
                    </button>
                </div>
            </label>
        {:else if resolver === 'k8s'}
            <div class="col-span-2 text-secondary-500">
                No additional configuration needed for K8s resolver.
            </div>
        {/if}
    </fieldset >
    {/if}
    <div class="justify-end flex">
        <button class="btn preset-filled-primary-500 " type="button" onclick={submit}>
            <WrenchIcon size={16}></WrenchIcon>
            Build
        </button>
    </div>
</form>