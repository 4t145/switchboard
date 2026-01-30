<script lang="ts">
	interface Props {
		mode?: 'horizontal' | 'overlap';
		class: string;
	}

	let { mode = 'horizontal', class: className = '' }: Props = $props();
</script>

{#if mode === 'horizontal'}
	<div class={className}>
		<span class="bg-cyan-500 px-[0.3em] text-white">S</span><span class="px-[0.1em] text-black"
			>witch</span
		>
		<span class="bg-pink-500 px-[0.3em] text-white">B</span><span class="px-[0.1em] text-black"
			>oard</span
		>
	</div>
{:else if mode === 'overlap'}
	<div class={'relative inline-block h-8 w-8 ' + className}>
		<div class="absolute top-0 left-0 h-6 w-6 bg-cyan-500 mix-blend-multiply"></div>
		<div class="absolute right-0 bottom-0 h-6 w-6 bg-pink-500 mix-blend-multiply"></div>
		<span
			class="absolute top-0 left-0 z-10 flex h-6 w-6 items-center justify-center font-bold text-white"
			>S</span
		>
		<span
			class="absolute right-0 bottom-0 z-10 flex h-6 w-6 items-center justify-center font-bold text-white"
			>B</span
		>
	</div>
{/if}
