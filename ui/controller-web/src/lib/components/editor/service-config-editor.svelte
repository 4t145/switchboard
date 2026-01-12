<script lang="ts">
    import { Trash2, Plus, Edit } from 'lucide-svelte';
    import type { ServiceConfig, TcpService, Listener, Tls, TcpRoute } from '$lib/api/types';
    import { Tabs } from '@skeletonlabs/skeleton-svelte';
    import TcpServiceForm from './tcp-service-form.svelte';
    import ListenerForm from './listener-form.svelte';
    import TlsConfigForm from './tls-config-form.svelte';
    import TcpRouteForm from './tcp-route-form.svelte';

    let { config = $bindable() } = $props<{ config: ServiceConfig }>();

    let activeTab = $state('services');
    let editingItem: { type: string, key: string } | null = $state(null);

    // Initializers for new items
    const initializers = {
        services: (): TcpService => ({ name: '', provider: 'Static', config: {}, description: '' }),
        listeners: (): Listener => ({ bind: '', description: '' }),
        tls: (): Tls => ({ 
            resolver: { Single: { certs: [], key: '', ocsp: null } } as any, 
            options: {
                ignore_client_order: false,
                max_fragment_size: null,
                alpn_protocols: [],
                enable_secret_extraction: false,
                max_early_data_size: 0,
                send_half_rtt_data: false,
                send_tls13_tickets: 0,
                require_ems: false
            } as any
        }),
        routes: (): TcpRoute => ({ bind: '', service: '', tls: '' })
    };

    function addItem(type: 'services' | 'listeners' | 'tls' | 'routes') {
        const prefix = type.slice(0, -1); // remove 's'
        let key = `${prefix}-${Object.keys(config[`tcp_${type}`] || config[type] || {}).length + 1}`;
        
        let counter = 1;
        const collection = type === 'services' ? config.tcp_services :
                          type === 'listeners' ? config.tcp_listeners :
                          type === 'routes' ? config.tcp_routes :
                          config.tls;
                          
        while (key in collection) {
            key = `${prefix}-${Object.keys(collection).length + ++counter}`;
        }

        if (type === 'services') config.tcp_services[key] = initializers.services();
        else if (type === 'listeners') config.tcp_listeners[key] = initializers.listeners();
        else if (type === 'tls') config.tls[key] = initializers.tls();
        else if (type === 'routes') config.tcp_routes[key] = initializers.routes();

        editingItem = { type, key };
    }

    function deleteItem(type: 'services' | 'listeners' | 'tls' | 'routes', key: string) {
        if (type === 'services') delete config.tcp_services[key];
        else if (type === 'listeners') delete config.tcp_listeners[key];
        else if (type === 'tls') delete config.tls[key];
        else if (type === 'routes') delete config.tcp_routes[key];
        
        if (editingItem?.type === type && editingItem?.key === key) {
            editingItem = null;
        }
        
        config = config; // trigger reactivity
    }

    function getKeys(obj: Record<string, any>) {
        return Object.keys(obj || {});
    }

    function renameItem(type: 'services' | 'listeners' | 'tls' | 'routes', oldKey: string, newKey: string) {
        if (!newKey || oldKey === newKey) return;
        
        const collection = getCollection(type);
        if (newKey in collection) {
            // alert('Key already exists!'); // Simple alert for now, or use a toast
            return;
        }

        const value = collection[oldKey];
        delete collection[oldKey];
        collection[newKey] = value;
        
        // Sync internal name for services
        if (type === 'services' && value && typeof value === 'object' && 'name' in value) {
            (value as TcpService).name = newKey;
        }

        if (editingItem?.type === type && editingItem?.key === oldKey) {
            editingItem.key = newKey;
        }
        
        config = config; // trigger reactivity
    }

    function getCollection(type: string) {
        if (type === 'services') return config.tcp_services;
        if (type === 'listeners') return config.tcp_listeners;
        if (type === 'tls') return config.tls;
        if (type === 'routes') return config.tcp_routes;
        return {};
    }
</script>

<div class="h-full flex flex-col p-4">
    <!-- Tabs Header -->
    <Tabs value={activeTab} onValueChange={(e) => activeTab = e.value}>
        <Tabs.List class="mb-4">
            <Tabs.Trigger value="services">Services</Tabs.Trigger>
            <Tabs.Trigger value="listeners">Listeners</Tabs.Trigger>
            <Tabs.Trigger value="tls">TLS</Tabs.Trigger>
            <Tabs.Trigger value="routes">Routes</Tabs.Trigger>
        </Tabs.List>
        
        <!-- Tab Content Wrapper - We use conditional rendering instead of multiple Tabs.Content to share layout -->
        <div class="h-[calc(100vh-12rem)] flex gap-6">
            
            <!-- Sidebar: List -->
            <div class="w-80 flex-none card bg-surface-100-800-token shadow-sm flex flex-col border border-surface-200-700-token">
                <div class="p-4 border-b border-surface-200-700-token flex justify-between items-center flex-none">
                    <h3 class="font-bold text-lg capitalize">{activeTab}</h3>
                    <button class="btn btn-sm variant-filled-primary" onclick={() => addItem(activeTab as any)}>
                        <Plus size={16} /> Add
                    </button>
                </div>
                <div class="overflow-y-auto flex-1 p-2 space-y-2">
                    {#each getKeys(getCollection(activeTab)) as key}
                        <!-- svelte-ignore a11y_click_events_have_key_events -->
                        <!-- svelte-ignore a11y_no_static_element_interactions -->
                        <div class="flex items-center justify-between p-3 rounded-container-token hover:bg-surface-200-700-token transition-colors cursor-pointer border border-transparent {editingItem?.key === key && editingItem?.type === activeTab ? 'variant-soft-primary' : ''}"
                             onclick={() => editingItem = { type: activeTab, key }}>
                            <span class="font-medium truncate flex-1">{key}</span>
                            <div class="flex gap-1">
                                <button class="btn-icon btn-icon-sm" onclick={(e) => { e.stopPropagation(); deleteItem(activeTab as any, key); }}>
                                    <Trash2 size={14} class="text-error-500" />
                                </button>
                            </div>
                        </div>
                    {/each}
                    {#if getKeys(getCollection(activeTab)).length === 0}
                        <div class="text-center p-8 opacity-50 text-sm">
                            No items found.<br/>Click "Add" to create one.
                        </div>
                    {/if}
                </div>
            </div>

            <!-- Editor: Detail -->
            <div class="flex-1 card bg-surface-100-800-token shadow-sm border border-surface-200-700-token flex flex-col overflow-hidden">
                {#if editingItem && editingItem.type === activeTab}
                    <div class="p-4 border-b border-surface-200-700-token bg-surface-200-700-token/50 flex justify-between items-center flex-none">
                        <div class="flex items-center gap-2">
                            <Edit size={16} class="opacity-70" />
                            <span class="font-bold">Editing: {editingItem.key}</span>
                        </div>
                    </div>
                    
                    <div class="overflow-y-auto flex-1">
                        {#if activeTab === 'services' && config.tcp_services[editingItem.key]}
                            <TcpServiceForm 
                                bind:value={config.tcp_services[editingItem.key]} 
                                key={editingItem.key}
                                onKeyChange={(newKey) => renameItem('services', editingItem!.key, newKey)}
                            />
                        {:else if activeTab === 'listeners' && config.tcp_listeners[editingItem.key]}
                            <ListenerForm 
                                bind:value={config.tcp_listeners[editingItem.key]} 
                                key={editingItem.key}
                                onKeyChange={(newKey) => renameItem('listeners', editingItem!.key, newKey)}
                            />
                        {:else if activeTab === 'tls' && config.tls[editingItem.key]}
                            <TlsConfigForm 
                                bind:value={config.tls[editingItem.key]} 
                                key={editingItem.key}
                                onKeyChange={(newKey) => renameItem('tls', editingItem!.key, newKey)}
                            />
                        {:else if activeTab === 'routes' && config.tcp_routes[editingItem.key]}
                            <TcpRouteForm 
                                bind:value={config.tcp_routes[editingItem.key]} 
                                key={editingItem.key}
                                onKeyChange={(newKey) => renameItem('routes', editingItem!.key, newKey)}
                                listenerKeys={getKeys(config.tcp_listeners)}
                                serviceKeys={getKeys(config.tcp_services)}
                                tlsKeys={getKeys(config.tls)}
                            />
                        {/if}
                    </div>
                {:else}
                    <div class="h-full flex flex-col items-center justify-center opacity-40">
                        <Edit size={48} class="mb-4" />
                        <p class="text-lg">Select an item to edit details</p>
                    </div>
                {/if}
            </div>
        </div>
    </Tabs>
</div>
