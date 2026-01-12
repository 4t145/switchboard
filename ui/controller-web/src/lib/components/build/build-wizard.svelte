<script lang="ts">
    import { Check, ChevronRight, Save } from 'lucide-svelte';
    import StepSourceSelection from './step-source-selection.svelte';
    import StepConfigEditor from './step-config-editor.svelte';
    import type { ServiceConfig } from '$lib/api/types';

    let currentStep = $state<number>(1);
    let config = $state<ServiceConfig | null>(null);
    let sourceSummary = $state<string>("");
    let saveAs = $state<string | undefined>(undefined);

    function onSourceSelected(loadedConfig: Record<string, any>, summary: string, saveAsVal?: string) {
        config = loadedConfig as ServiceConfig;
        sourceSummary = summary;
        saveAs = saveAsVal;
        currentStep = 2;
    }

    function goToStep(step: number) {
        if (currentStep < step) return; // Prevent jumping forward
        currentStep = step;
    }
    
    function handleSave() {
        console.log("Saving config...", config);
        // TODO: Implement save logic
        alert("Save/Deploy logic not implemented yet.\nCheck console for config object.");
    }
</script>

<div class="h-full flex flex-col">
    
    <!-- Top Bar Stepper -->
    <div class="flex-none flex items-center justify-between p-4 border-b border-surface-200 dark:border-surface-700 bg-surface-50 dark:bg-surface-900">
        <div class="flex items-center gap-2 overflow-x-auto">
            
            <!-- Step 1 Indicator -->
            <button class="flex items-center gap-2 px-3 py-2 rounded transition-colors
                {currentStep === 1 ? 'bg-primary-100 dark:bg-primary-900/30 text-primary-700 dark:text-primary-300 font-bold' : 
                 currentStep > 1 ? 'hover:bg-surface-200 dark:hover:bg-surface-800 text-surface-600 dark:text-surface-300' : 'opacity-50'}"
                 onclick={() => goToStep(1)}>
                <div class="w-6 h-6 rounded-full flex items-center justify-center text-xs border
                    {currentStep >= 1 ? 'border-primary-500 bg-primary-500 text-white' : 'border-surface-400'}">
                    {#if currentStep > 1} <Check size={14} /> {:else} 1 {/if}
                </div>
                <span class="whitespace-nowrap">Select Source</span>
            </button>

            <ChevronRight size={16} class="text-surface-400" />

            <!-- Step 2 Indicator -->
            <button class="flex items-center gap-2 px-3 py-2 rounded transition-colors
                {currentStep === 2 ? 'bg-primary-100 dark:bg-primary-900/30 text-primary-700 dark:text-primary-300 font-bold' : 
                 currentStep > 2 ? 'hover:bg-surface-200 dark:hover:bg-surface-800 text-surface-600 dark:text-surface-300' : 
                 'text-surface-400 dark:text-surface-500 cursor-not-allowed'}"
                 onclick={() => goToStep(2)}
                 disabled={currentStep < 2}>
                <div class="w-6 h-6 rounded-full flex items-center justify-center text-xs border
                    {currentStep >= 2 ? 'border-primary-500 bg-primary-500 text-white' : 'border-surface-400'}">
                    2
                </div>
                <span class="whitespace-nowrap">Edit Configuration</span>
            </button>

            <!-- Optional Summary in Header -->
            {#if currentStep > 1 && sourceSummary}
                <div class="hidden md:flex items-center ml-4 pl-4 border-l border-surface-300 dark:border-surface-600">
                     <span class="text-xs text-surface-500 mr-2">Source:</span>
                     <span class="badge variant-soft-secondary text-xs">{sourceSummary}</span>
                </div>
            {/if}
        </div>

        <!-- Right Actions -->
        <div class="flex items-center gap-2">
            {#if currentStep === 2}
                <button class="btn btn-sm variant-filled-primary" onclick={handleSave}>
                    <Save size={16} class="mr-2" />
                    Save / Deploy
                </button>
            {/if}
        </div>
    </div>

    <!-- Main Content Area -->
    <div class="flex-1 overflow-hidden relative">
        
        <!-- Step 1 Content -->
        <!-- Using visibility/display toggling via absolute positioning to preserve state -->
        <div class="absolute inset-0 overflow-auto p-6 transition-opacity duration-300 bg-surface-50 dark:bg-surface-900"
             class:hidden={currentStep !== 1}>
             <div class="max-w-6xl mx-auto h-full flex flex-col justify-center">
                 <div class="mb-8 text-center">
                     <h2 class="h2 font-bold mb-2">How would you like to start?</h2>
                     <p class="text-surface-500">Select a source to load your initial configuration from.</p>
                 </div>
                 <StepSourceSelection onNext={onSourceSelected} />
             </div>
        </div>

        <!-- Step 2 Content -->
        <!-- Conditional rendering for Step 2 is fine as it depends on config loaded in Step 1 -->
        {#if currentStep === 2 && config}
            <div class="absolute inset-0 overflow-hidden bg-surface-50 dark:bg-surface-900 p-4">
                 <StepConfigEditor bind:config={config} />
            </div>
        {/if}

    </div>

</div>
