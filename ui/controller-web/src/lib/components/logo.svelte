<script lang="ts">
	interface Props {
		mode?: 'horizontal' | 'overlap';
	}
	
	let { mode = 'horizontal' }: Props = $props();
</script>

{#if mode === 'horizontal'}
	<span class="bg-cyan-500 px-[0.3em] text-white">S</span><span class="px-[0.1em] text-black">witch</span>
	<span class="bg-pink-500 px-[0.3em] text-white">B</span><span class="px-[0.1em] text-black">oard</span>
{:else if mode === 'overlap'}
	<div class="relative inline-block h-8 w-8">
		<div class="absolute left-0 top-0 h-6 w-6 bg-cyan-500 mix-blend-multiply"></div>
		<div class="absolute bottom-0 right-0 h-6 w-6 bg-pink-500 mix-blend-multiply"></div>
		<span class="absolute left-0 top-0 z-10 flex h-6 w-6 items-center justify-center text-white font-bold">S</span>
		<span class="absolute bottom-0 right-0 z-10 flex h-6 w-6 items-center justify-center text-white font-bold">B</span>
	</div>
{/if}
