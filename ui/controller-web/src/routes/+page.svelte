<script lang="ts">
	import { goto } from '$app/navigation';
</script>

<div class="min-h-screen bg-gradient-to-br from-blue-50 to-indigo-100 flex items-center justify-center">
	<div class="max-w-md w-full space-y-8 p-8">
		<div class="text-center">
			<div class="mx-auto h-16 w-16 bg-blue-600 rounded-lg flex items-center justify-center mb-4">
				<span class="text-white font-bold text-xl">SW</span>
			</div>
			<h2 class="text-3xl font-extrabold text-gray-900">
				Switchboard
			</h2>
			<p class="mt-2 text-sm text-gray-600">
				网关管理平台
			</p>
		</div>
		
		<div class="space-y-4">
			<button 
				onclick={() => goto('/admin')}
				class="group relative w-full flex justify-center py-3 px-4 border border-transparent text-sm font-medium rounded-md text-white bg-blue-600 hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500 transition-colors"
			>
				进入管理面板
			</button>
		</div>

		<div class="text-center text-xs text-gray-500">
			版本 1.0.0 - 基于 Rust & SvelteKit
		</div>
	</div>
</div>
