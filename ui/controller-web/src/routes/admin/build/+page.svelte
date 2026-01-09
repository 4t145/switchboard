<script lang="ts">
	import { api } from "$lib/api/routes";
	import type { ResolveServiceConfigRequest } from "$lib/api/routes/resolve";
	import { ResolveServiceConfigForm } from "$lib/components";

    let resolver = $state('');
    let config = $state<Record<string, any> | null>(null);
    let saveAs = $state<string | null>(null);
    let source = $state<"environment" | "create_new" | "other">("environment");
    async function resolveConfig(request: ResolveServiceConfigRequest) {
        const response = await api.resolve.service_config(request);
        config = response.config;
    }
</script>

<h1>
    Build 
</h1>

<ResolveServiceConfigForm 
    bind:resolver
    bind:config
    bind:saveAs
    onSubmit={(request) => {
        console.log("Submitted request:", request);
    }}
>
</ResolveServiceConfigForm>