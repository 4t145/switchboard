import type { ProviderEditorPlugin } from "$lib/plugins/types";
import PortForwardEditor from "./port-forward-editor.svelte";

export type PortForwardConfig = {
    // socket addr
    to: string;
}

export const portForwardEditorPlugin: ProviderEditorPlugin<PortForwardConfig> = {
    provider: 'pf',
    displayName: 'Port Forwarding Service',
    component: PortForwardEditor,

    createDefaultConfig(): PortForwardConfig {
        return {
            to: ''
        };
    },

};

