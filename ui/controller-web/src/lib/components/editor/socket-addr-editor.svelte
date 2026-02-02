<script lang="ts">
	import type { HTMLInputAttributes } from "svelte/elements";

    type Props = HTMLInputAttributes;
    let { value = $bindable(), ...props}: Props = $props();
    
    type IpAddr = {
        kind: 'v4' | 'v6',
        address: string;
    }
    type SocketAddr = {
        ip: IpAddr;
        port: number;
    }
    type ParseSocketAddrError = {
        reason: string;
        error: string;
    }
    function formatSocketAddr(addr: SocketAddr): string {
        if (addr.ip.kind === 'v6') {
            return `[${addr.ip.address}]:${addr.port}`;
        } else {
            return `${addr.ip.address}:${addr.port}`;
        }
    }
    function parseSocketAddr(input: string): SocketAddr | ParseSocketAddrError {
        // ipv6: [<address>]:<port>
        if (input.startsWith('[')) {
            const closingBracketIndex = input.indexOf(']');
            if (closingBracketIndex === -1) {
                return { reason: 'invalid_format', error: 'Missing closing bracket for IPv6 address' };
            }
            const ipStr = input.slice(1, closingBracketIndex);
            const portStr = input.slice(closingBracketIndex + 2); // Skip ]:
            const port = Number(portStr);
            if (isNaN(port) || port < 1 || port > 65535) {
                return { reason: 'invalid_port', error: 'Port must be a number between 1 and 65535' };
            }
            if (!/^([0-9a-fA-F]{0,4}:){2,7}[0-9a-fA-F]{0,4}$/.test(ipStr)) {
                return { reason: 'invalid_ip', error: 'IP address is not valid' };
            }
            return {
                ip: { kind: 'v6', address: ipStr },
                port
            };
        } else {
            // ipv4: <address>:<port>
            // find last colon
            const lastColonIndex = input.lastIndexOf(':');
            if (lastColonIndex === -1) {
                return { reason: 'invalid_format', error: 'Address must be in the format IP:PORT' };
            }
            const ipStr = input.slice(0, lastColonIndex);
            const portStr = input.slice(lastColonIndex + 1);
            const port = Number(portStr);
            if (isNaN(port) || port < 1 || port > 65535) {
                return { reason: 'invalid_port', error: 'Port must be a number between 1 and 65535' };
            }
            if (/^(\d{1,3}\.){3}\d{1,3}$/.test(ipStr)) {
                return {
                    ip: { kind: 'v4', address: ipStr },
                    port
                };
            } else {
                return { reason: 'invalid_ip', error: 'IP address is not valid' };
            }
        }
    }
    function isSocketAddr( result: SocketAddr | ParseSocketAddrError ): result is SocketAddr {
        return !('reason' in result);
    }
    let validationError = $state<ParseSocketAddrError | null>(null);
    function onblur() {
        const result = parseSocketAddr(value as string);
        if (isSocketAddr(result)) {
            // Normalize the input value
            value = formatSocketAddr(result);
            inputElement.setCustomValidity('');
            validationError = null;
        } else {
            validationError = result;
            inputElement.setCustomValidity(result.error);
        }
    }
    let inputElement: HTMLInputElement;
</script>

<input type="text" class={`input-bordered input w-full font-mono ${validationError ? 'bg-error-100 dark:bg-error-900' : ''}`}  {onblur} bind:value bind:this={inputElement} {...props}/>
{#if validationError}
    <div class="text-xs text-error-600 dark:text-error-400 mt-1">
        {validationError.error}
    </div>
{/if}