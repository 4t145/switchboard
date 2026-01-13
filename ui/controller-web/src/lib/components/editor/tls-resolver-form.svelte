<script lang="ts">
    import FileToBase64Input from '$lib/components/common/file-to-base64-input.svelte';
    import { Trash2, Plus } from 'lucide-svelte';

    type TlsCertParams = {
        certs: string[]; // Base64
        key: string;     // Base64
        ocsp: string | null; // Base64
    };

    // TlsResolver can be { Single: ... } or { Sni: ... }
    // We bind to the whole object.
    let { 
        value = $bindable() 
    } = $props<{ 
        value: { Single?: TlsCertParams; Sni?: Record<string, TlsCertParams> } 
    }>();

    // Internal state to track mode
    let mode = $state<'Single' | 'Sni'>('Single');
    
    // Initialize
    $effect(() => {
        if (value.Sni) mode = 'Sni';
        else mode = 'Single';
    });

    function switchMode(newMode: 'Single' | 'Sni') {
        mode = newMode;
        if (newMode === 'Single') {
            value = { Single: { certs: [], key: '', ocsp: null } };
        } else {
            value = { Sni: {} };
        }
    }

    // --- Helpers for Cert Params ---
    function addCert(params: TlsCertParams) {
        params.certs = [...params.certs, ''];
    }

    function removeCert(params: TlsCertParams, index: number) {
        params.certs = params.certs.filter((_, i) => i !== index);
    }

</script>

{#snippet certForm(params: TlsCertParams)}
    <div class="space-y-4 p-4 border border-surface-200 dark:border-surface-700 rounded bg-surface-50 dark:bg-surface-900/50">
        <!-- Key -->
        <FileToBase64Input 
            label="Private Key (PEM)" 
            bind:value={params.key} 
            accept=".key,.pem"
            helperText="Upload the private key file."
        />

        <!-- Certs -->
        <div class="space-y-2">
            <div class="flex justify-between items-center">
                <span class="label-text font-bold">Certificates Chain</span>
                <button class="btn btn-sm variant-ghost-primary" onclick={() => addCert(params)}>
                    <Plus size={14} /> Add Cert
                </button>
            </div>
            
            {#if params.certs.length === 0}
                <div class="text-sm opacity-50 italic">No certificates added.</div>
            {/if}

            {#each params.certs as _, i}
                <div class="flex gap-2 items-start">
                    <div class="flex-1">
                        <FileToBase64Input 
                            label={`Certificate ${i + 1}`} 
                            bind:value={params.certs[i]} 
                            accept=".crt,.pem,.cer"
                        />
                    </div>
                    <button class="btn-icon btn-icon-sm variant-soft-error mt-8" onclick={() => removeCert(params, i)}>
                        <Trash2 size={16} />
                    </button>
                </div>
            {/each}
        </div>

        <!-- OCSP -->
        <!-- <FileToBase64Input 
            label="OCSP Stapling (Optional)" 
            bind:value={params.ocsp as string} 
            accept=".der,.ocsp"
            helperText="Upload OCSP response if needed."
        /> -->
    </div>
{/snippet}


<div class="space-y-4">
    <!-- Mode Switcher -->
    <div class="flex gap-4 border-b border-surface-200 dark:border-surface-700">
        <button 
            class="px-4 py-2 border-b-2 transition-colors {mode === 'Single' ? 'border-primary-500 font-bold text-primary-500' : 'border-transparent opacity-50'}"
            onclick={() => switchMode('Single')}
        >
            Single Certificate
        </button>
        <button 
            class="px-4 py-2 border-b-2 transition-colors {mode === 'Sni' ? 'border-primary-500 font-bold text-primary-500' : 'border-transparent opacity-50'}"
            onclick={() => switchMode('Sni')}
        >
            SNI Mapping
        </button>
    </div>

    {#if mode === 'Single' && value.Single}
        {@render certForm(value.Single)}
    {:else if mode === 'Sni' && value.Sni}
        <!-- Sni Editor -->
        <div class="space-y-4">
             {#each Object.entries(value.Sni) as [domain, params]}
                <div class="card p-4 border border-surface-300 dark:border-surface-600">
                    <div class="flex justify-between items-center mb-4">
                         <h4 class="h4 font-bold">{domain}</h4>
                         <button class="btn btn-sm variant-filled-error" onclick={() => {
                             const newSni = { ...value.Sni };
                             delete newSni[domain];
                             value.Sni = newSni;
                         }}>Delete</button>
                    </div>
                    {@render certForm(params as TlsCertParams)}
                </div>
             {/each}

             <!-- Add Domain -->
             <div class="flex gap-2 items-end">
                 <label class="label">
                     <span>New Domain (SNI)</span>
                     <input class="input" type="text" placeholder="example.com" id="new-sni-domain" />
                 </label>
                 <button class="btn variant-filled-primary" onclick={() => {
                     const input = document.getElementById('new-sni-domain') as HTMLInputElement;
                     const domain = input.value;
                     if (domain && value.Sni) {
                         value.Sni = { 
                             ...value.Sni, 
                             [domain]: { certs: [], key: '', ocsp: null } 
                         };
                         input.value = '';
                     }
                 }}>Add Domain</button>
             </div>
        </div>
    {/if}
</div>
