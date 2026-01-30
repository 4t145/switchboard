<script lang="ts">
	import { goto } from '$app/navigation';
</script>

<div
	class="flex min-h-screen items-center justify-center bg-gradient-to-br from-blue-50 to-indigo-100"
>
	<div class="w-full max-w-md space-y-8 p-8">
		<div class="text-center">
			<div class="mx-auto mb-4 flex h-16 w-16 items-center justify-center rounded-lg bg-blue-600">
				<span class="text-xl font-bold text-white">SW</span>
			</div>
			<h2 class="text-3xl font-extrabold text-gray-900">Switchboard</h2>
			<p class="mt-2 text-sm text-gray-600">网关管理平台</p>
		</div>

		<div class="space-y-4">
			<button
				onclick={() => goto('/admin')}
				class="group relative flex w-full justify-center rounded-md border border-transparent bg-blue-600 px-4 py-3 text-sm font-medium text-white transition-colors hover:bg-blue-700 focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 focus:outline-none"
			>
				进入管理面板
			</button>
		</div>

		<div class="text-center text-xs text-gray-500">版本 1.0.0 - 基于 Rust & SvelteKit</div>
	</div>
</div>
