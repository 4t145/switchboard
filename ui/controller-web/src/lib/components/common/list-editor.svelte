<script lang="ts" generics="T extends Record<string, any>">
	import { ArrowLeft, Edit2, Plus, Search, Trash2, X } from "lucide-svelte";
	import type { Snippet } from "svelte";

	interface Props {
		items: T[];
		title: string;
		getItemName: (item: T) => string;
		createItem: () => T;
		renderEditor: Snippet<[T, number]>;
		emptyMessage?: string;

		// 新增响应式配置
		responsive?: boolean;
		sidebarWidth?: string;
		breakpoint?: number;
		mobileMode?: 'stack' | 'modal';
	}

	let {
		items = $bindable(),
		title,
		getItemName,
		createItem,
		renderEditor,
		emptyMessage,

		// 响应式配置 - 默认值
		responsive = true,
		sidebarWidth = 'w-80',
		breakpoint = 768,
		mobileMode = 'stack'
	}: Props = $props();
	
	let editingIndex = $state<number | null>(null);
	let searchQuery = $state<string>('');

	// 响应式状态
	let isMobile = $state(false);
	let showMobileEditor = $state(false);

	// 过滤后的项目列表
	let filteredItems = $derived(
		searchQuery.trim() === ''
			? items.map((item, index) => ({ item, originalIndex: index }))
			: items
				.map((item, index) => ({ item, originalIndex: index }))
				.filter(({ item }) =>
					getItemName(item).toLowerCase().includes(searchQuery.toLowerCase())
				)
	);

	// 响应式检测
	function updateLayout() {
		if (!responsive) return;
		
		isMobile = window.innerWidth < breakpoint;
		if (!isMobile) {
			showMobileEditor = false;
		}
	}

	// 响应式监听器
	$effect(() => {
		if (!responsive) return;
		
		function handleResize() {
			updateLayout();
		}
		
		updateLayout();
		window.addEventListener('resize', handleResize);
		
		return () => {
			window.removeEventListener('resize', handleResize);
		};
	});

	// 自动选择第一个项目
	$effect(() => {
		if (editingIndex === null && items.length > 0) {
			editingIndex = 0;
		}
	});

	function addItem() {
		const newItem = createItem();
		items = [...items, newItem];
		editingIndex = items.length - 1;
		
		// 移动端显示编辑器
		if (isMobile) {
			showMobileEditor = true;
		}
	}

	function selectItem(index: number) {
		editingIndex = index;
		
		// 移动端切换到编辑器
		if (isMobile) {
			showMobileEditor = true;
		}
	}

	function backToList() {
		showMobileEditor = false;
	}

	function removeItem(index: number) {
		if (index === editingIndex) {
			editingIndex = index > 0 ? index - 1 : items.length > 1 ? 0 : null;
		} else if (editingIndex !== null && index < editingIndex) {
			editingIndex = editingIndex - 1;
		}
		items = items.filter((_, i) => i !== index);
	}

	let emptyDisplayMessage = $derived(emptyMessage || `No ${title.toLowerCase()}`);
</script>

<!-- 主容器 -->
<div class="flex h-full flex-col {responsive ? '' : 'md:flex-row'}">
	<!-- 头部区域 -->
	<header class="flex-none border-b border-surface-200-700 p-4">
		<h2 class="text-lg font-semibold text-surface-900-100">{title}</h2>
		<div class="mt-2">
			<div class="relative">
				<Search class="absolute left-3 top-1/2 -translate-y-1/2 text-surface-500" size={16} />
				<input
					class="input w-full pl-10"
					placeholder={`搜索 ${title.toLowerCase()}...`}
					bind:value={searchQuery}
				/>
				{#if searchQuery}
					<button
						class="btn-icon btn-icon-sm absolute right-1 top-1/2 -translate-y-1/2"
						onclick={() => (searchQuery = '')}
					>
						<X size={14} />
					</button>
				{/if}
			</div>
		</div>
	</header>

	<!-- 内容区域 -->
	<div class="flex min-h-0 flex-1 {responsive && isMobile ? 'flex-col' : ''}">
		<!-- 侧边栏/列表区域 -->
		<aside class="flex-none overflow-y-auto bg-surface-50-950 {responsive && isMobile ? 'flex-1' : sidebarWidth}">
			<div class="flex-1 space-y-2 overflow-y-auto p-2">
				{#each filteredItems as { item, originalIndex } (originalIndex)}
					<!-- svelte-ignore a11y_click_events_have_key_events -->
					<!-- svelte-ignore a11y_no_static_element_interactions -->
					<div
						class="rounded-container-token hover:preset-tonal flex cursor-pointer items-center justify-between p-3 transition-all min-h-12
						       {editingIndex === originalIndex ? 'preset-tonal-primary' : ''}"
						onclick={() => selectItem(originalIndex)}
						onkeydown={(e) => {
							if (e.key === 'Enter' || e.key === ' ') {
								e.preventDefault();
								selectItem(originalIndex);
							}
						}}
					>
						<span class="flex-1 truncate font-medium text-surface-900-100">
							{getItemName(item)}
						</span>
						<div class="flex gap-1">
							<button
								class="btn-icon btn-icon-sm"
								onclick={(e) => {
									e.stopPropagation();
									removeItem(originalIndex);
								}}
							>
								<Trash2 size={14} class="text-error-500" />
							</button>
						</div>
					</div>
				{/each}
				
				{#if filteredItems.length === 0 && items.length > 0}
					<div class="p-8 text-center text-sm text-surface-500">
						未找到匹配项 "{searchQuery}"
					</div>
				{:else if items.length === 0}
					<div class="p-8 text-center text-surface-500">
						{emptyDisplayMessage}
					</div>
				{/if}
			</div>

			{#if items.length > 0}
				<div class="border-t border-surface-200-700 p-2">
					<button
						onclick={addItem}
						class="btn btn-sm preset-tonal w-full"
					>
						<Plus size={16} class="mr-2" />
						添加 {title.slice(0, -1)}
					</button>
				</div>
			{:else}
				<div class="p-2">
					<button
						onclick={addItem}
						class="btn btn-sm preset-filled-primary w-full"
					>
						<Plus size={16} class="mr-2" />
						创建第一个 {title.slice(0, -1)}
					</button>
				</div>
			{/if}
		</aside>

		<!-- 编辑器区域 -->
		<main class="flex flex-1 flex-col overflow-hidden bg-surface-0-1000 
		           {responsive && isMobile && !showMobileEditor ? 'hidden' : ''}">
			{#if editingIndex !== null && items[editingIndex]}
				{#if responsive && isMobile && showMobileEditor}
					<!-- 移动端编辑器头部 -->
					<div class="flex items-center gap-2 border-b border-surface-200-700 p-4">
						<button
							class="btn btn-sm preset-tonal"
							onclick={backToList}
						>
							<ArrowLeft size={16} class="mr-2" />
							返回
						</button>
						<span class="font-medium text-surface-900-100 truncate">
							{getItemName(items[editingIndex])}
						</span>
					</div>
				{/if}

				<!-- 编辑器内容 -->
				<div class="flex-1 overflow-y-auto">
					{#key editingIndex}
						{@render renderEditor(items[editingIndex], editingIndex)}
					{/key}
				</div>
			{:else if items.length === 0}
				<div class="flex flex-1 items-center justify-center">
					<div class="text-center">
						<div class="text-surface-500 mb-4">
							<Edit2 size={48} class="mx-auto mb-2" />
							<p class="text-lg font-medium">暂无{title}</p>
							<p class="text-sm">点击上方按钮创建第一个{title.slice(0, -1)}</p>
						</div>
					</div>
				</div>
			{:else if filteredItems.length === 0}
				<div class="flex flex-1 items-center justify-center">
					<div class="text-center text-surface-500">
						<Search size={48} class="mx-auto mb-2" />
						<p class="text-lg font-medium">未找到匹配项</p>
						<p class="text-sm">尝试修改搜索条件</p>
					</div>
				</div>
			{:else}
				<div class="flex flex-1 items-center justify-center">
					<div class="text-center text-surface-500">
						<Edit2 size={48} class="mx-auto mb-2" />
						<p class="text-lg font-medium">请选择一个{title.slice(0, -1)}</p>
						<p class="text-sm">从左侧列表中选择要编辑的项目</p>
					</div>
				</div>
			{/if}
		</main>
	</div>
</div>

<style>
	/* 确保触摸目标足够大 */
	@media (max-width: 768px) {
		button {
			min-height: 44px;
			min-width: 44px;
		}
	}
</style>