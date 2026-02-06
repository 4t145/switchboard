<script lang="ts">
	import TableListEditor, {
		type RowParams,
		type ListOperations
	} from '$lib/components/common/table-list-editor.svelte';
	import type { HttpClassEditorProps } from '../';
	import { CrossIcon, DeleteIcon, PlusIcon } from '@lucide/svelte';
	type HeaderPair = [string, string];
	export type StaticResponseConfig = {
		headers: HeaderPair[];
		status_code: number;
		body: string | null;
	};

	type Props = HttpClassEditorProps<StaticResponseConfig>;

	let { value = $bindable(), readonly = false }: Props = $props();

    function isValidStatusCode(code: number): boolean {
        return Number.isInteger(code) && code >= 100 && code <= 599;
    }
    function isValidHeaderName(name: string): boolean {
        // Simple validation: header name should not be empty and should not contain control characters
        return name.length > 0 && !/[\u0000-\u001F\u007F]/.test(name);
    }
    function isValidHeaderValue(value: string): boolean {
        // Simple validation: header value should not contain control characters
        return !/[\u0000-\u001F\u007F]/.test(value);
    }

</script>

<div class="space-y-4">
	<!-- Status Code -->
	<label class="label">
		<span class="label-text">Status Code</span>
		<input
			class="input-sm input"
			type="number"
			max="599"
			min="100"
			value={value.status_code}
            onchange={(e) => {
                value.status_code = Number(e.currentTarget.value);
            }}
            onblur={(e) => {
                const code = Number(e.currentTarget.value);
                if (!isValidStatusCode(code)) {
                    e.currentTarget.setCustomValidity('Status code must be an integer between 100 and 599');
                } else {
                    e.currentTarget.setCustomValidity('');
                }
                e.currentTarget.reportValidity();
            }}
			placeholder="200"
			disabled={readonly}
		/>
	</label>

	<!-- Headers -->
	<label class="label">
		<span class="label-text">Headers ({value.headers.length})</span>
        {#snippet header(ops: ListOperations<HeaderPair>)}
			<tr>
				<th class="w-1/2">Header Name</th>
				<th class="w-1/2">Header Value</th>
				<th class="w-0"></th>
			</tr>
		{/snippet}
		{#snippet row({ value, updateItem, deleteItem }: RowParams<HeaderPair>)}
			<tr>
				<td>
					<input
						class="input-sm input w-full"
						type="text"
						value={value[0]}
						onchange={(e) => updateItem([e.currentTarget.value, value[1]])}
                        onblur={(e) => {
                            if (!isValidHeaderName(e.currentTarget.value)) {
                                e.currentTarget.setCustomValidity('Invalid header name');
                            } else {
                                e.currentTarget.setCustomValidity('');
                            }
                            e.currentTarget.reportValidity();
                        }}
						placeholder="Header Name"
						disabled={readonly}
					/>
				</td>
				<td>
					<input
						class="input-sm input w-full"
						type="text"
						value={value[1]}
						onchange={(e) => updateItem([value[0], e.currentTarget.value])}
                        onblur={(e) => {
                            if (!isValidHeaderValue(e.currentTarget.value)) {
                                e.currentTarget.setCustomValidity('Invalid header value');
                            } else {
                                e.currentTarget.setCustomValidity('');
                            }
                            e.currentTarget.reportValidity();
                        }}
                        placeholder="Header Value"
						disabled={readonly}
					/>
				</td>
				<td class="w-0">
					{#if !readonly}
						<button
							class="btn-icon preset-outlined-error-500"
							type="button"
							onclick={() => deleteItem()}
							title="Delete Header"
						>
							<DeleteIcon></DeleteIcon>
						</button>
					{/if}
				</td>
			</tr>
		{/snippet}

		{#snippet footer(ops: ListOperations<HeaderPair>)}
			<tr>
				<td colspan="3" class="text-end">
					<button
						type="button"
						class="btn preset-outlined-primary-500 btn-sm"
						onclick={() => ops.addNewItem(['', ''])}
					>
						<PlusIcon class="size-4"></PlusIcon>Add Header</button
					>
				</td>
			</tr>
		{/snippet}
		<TableListEditor
			{row}
			{header}
			{footer}
			value={value.headers}
			onChange={(newHeaders) => (value.headers = newHeaders)}
		></TableListEditor>
	</label>

	<!-- Body -->
	<label class="label">
		<span class="label-text">Body</span>
		<textarea
			class="textarea-sm textarea w-full"
			value={value.body}
			onchange={(e) => (value.body = e.currentTarget.value)}
			placeholder="Response Body"
			rows={5}
			disabled={readonly}
		></textarea>
	</label>
</div>
