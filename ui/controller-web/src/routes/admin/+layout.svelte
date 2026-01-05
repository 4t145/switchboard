<script lang="ts">
	import { Collapsible, Button, NavigationMenu, DropdownMenu, Toggle } from 'bits-ui';
	import { page } from '$app/state';
	import { goto } from '$app/navigation';
	import {
		ChevronLeft,
		ChevronDown,
		Menu,
		Home,
		Activity,
		Users,
		Database,
		Settings,
		Moon,
		Sun,
		Globe,
		Bell,
		User
	} from 'lucide-svelte';
	import Logo from '$lib/components/logo.svelte';
	let { children } = $props();

	let isCollapsed = $state(false);
	let isMobileMenuOpen = $state(false);
	let isDarkMode = $state(false);
	let currentLanguage = $state('zh');

	// 语言选项
	const languages = [
		{ code: 'zh', name: '中文', flag: '🇨🇳' },
		{ code: 'en', name: 'English', flag: '🇺🇸' },
		{ code: 'ja', name: '日本語', flag: '🇯🇵' },
		{ code: 'ko', name: '한국어', flag: '🇰🇷' }
	];

	// 图标组件映射
	const iconComponents = {
		chevronLeft: ChevronLeft,
		chevronDown: ChevronDown,
		menu: Menu,
		home: Home,
		activity: Activity,
		users: Users,
		database: Database,
		settings: Settings,
		moon: Moon,
		sun: Sun,
		globe: Globe,
		bell: Bell,
		user: User
	};

	const navigationItems: {
		label: string;
		icon: keyof typeof iconComponents;
		href: string;
		subItems?: { label: string; href: string }[];
	}[] = [
		{ label: 'Dashboard', icon: 'home', href: '/admin' },
		{
			label: 'Services',
			icon: 'activity',
			href: '/admin/instances',
			subItems: [
				{ label: 'All Services', href: '/admin/instances' },
				{ label: 'Health Checks', href: '/admin/instances/health' },
				{ label: 'Route Builder', href: '/admin/instances/builder' }, // 新增
				{ label: 'Logs', href: '/admin/instances/logs' }
			]
		},
		{ label: 'Users', icon: 'users', href: '/admin/users' },
		{ label: 'Database', icon: 'database', href: '/admin/storage' },
		{ label: 'Settings', icon: 'settings', href: '/admin/settings' }
	];

	// 计算当前活跃路径
	let currentPath = $derived(page.url.pathname);

	function toggleSidebar() {
		isCollapsed = !isCollapsed;
	}

	function toggleMobileMenu() {
		isMobileMenuOpen = !isMobileMenuOpen;
	}

	function navigateTo(href: string) {
		goto(href);
		isMobileMenuOpen = false;
	}

	function isActive(href: string): boolean {
		return currentPath === href || currentPath.startsWith(href + '/');
	}

	function toggleDarkMode() {
		isDarkMode = !isDarkMode;
		// 这里可以添加实际的主题切换逻辑
		if (isDarkMode) {
			document.documentElement.classList.add('dark');
		} else {
			document.documentElement.classList.remove('dark');
		}
	}

	function changeLanguage(langCode: string) {
		currentLanguage = langCode;
		// 这里可以添加实际的语言切换逻辑
		console.log('Language changed to:', langCode);
	}
</script>

<div class="flex min-h-screen bg-gray-50 dark:bg-gray-900">
	<!-- Mobile menu backdrop -->
	{#if isMobileMenuOpen}
		<button
			aria-label="toggle mobile menu"
			class="fixed inset-0 z-40 bg-black bg-opacity-50 lg:hidden"
			onclick={toggleMobileMenu}
		></button>
	{/if}

	<!-- Sidebar -->
	<div
		class={`
        fixed inset-y-0 left-0 z-50 lg:relative 
        ${isCollapsed ? 'w-16' : 'w-64'} 
        ${isMobileMenuOpen ? 'translate-x-0' : '-translate-x-full lg:translate-x-0'}
        transform border-r border-gray-200 bg-white
        shadow-lg transition-all duration-300 ease-in-out dark:border-gray-700 dark:bg-gray-800
    `}
	>
		<!-- Sidebar Header -->
		<div
			class="flex h-16 items-center justify-between border-b border-gray-200 px-4 dark:border-gray-700"
		>
			{#if !isCollapsed}
				<div class="flex items-center space-x-2">
					<span class="text-lg font-semibold text-gray-900 dark:text-gray-100"><Logo /></span>
				</div>
			{:else}
				<div class="mx-auto flex h-8 w-8 items-center justify-center rounded-lg">
					<span class="text-lg font-semibold text-gray-900 dark:text-gray-100"
						><Logo mode="overlap" /></span
					>
				</div>
			{/if}

			<!-- Collapse toggle for desktop -->
			<Button.Root
				onclick={toggleSidebar}
				class="hidden rounded-md p-1 transition-colors hover:bg-gray-100 lg:flex dark:hover:bg-gray-700"
			>
				<div class={`transform transition-transform ${isCollapsed ? 'rotate-180' : ''}`}>
					<ChevronLeft class="h-4 w-4 text-gray-500 dark:text-gray-400" />
				</div>
			</Button.Root>
		</div>

		<!-- Navigation using NavigationMenu from bits-ui -->
		<NavigationMenu.Root class="flex-1 space-y-1 px-2 py-4">
			<NavigationMenu.List class="space-y-1">
				{#each navigationItems as item}
					{@const ItemIcon = iconComponents[item.icon]}
					<NavigationMenu.Item>
						{#if item.subItems}
							<!-- Collapsible menu item with subitems -->
							<Collapsible.Root>
								<Collapsible.Trigger
									class={`
                                        flex w-full items-center justify-between rounded-md px-3 py-2 text-sm transition-colors
                                        ${
																					isActive(item.href)
																						? 'border-r-2 border-blue-700 bg-blue-50 text-blue-700 dark:border-blue-400 dark:bg-blue-900/20 dark:text-blue-300'
																						: 'text-gray-700 hover:bg-gray-100 hover:text-gray-900 dark:text-gray-300 dark:hover:bg-gray-700 dark:hover:text-gray-100'
																				}
                                    `}
								>
									<div class="flex items-center space-x-3">
										<div class="flex-shrink-0">
											<ItemIcon class="h-5 w-5" />
										</div>
										{#if !isCollapsed}
											<span class="font-medium">{item.label}</span>
										{/if}
									</div>
									{#if !isCollapsed}
										<div class="flex-shrink-0">
											<ChevronDown class="h-4 w-4 text-gray-500 dark:text-gray-400" />
										</div>
									{/if}
								</Collapsible.Trigger>

								{#if !isCollapsed}
									<Collapsible.Content class="ml-6 mt-1 space-y-1">
										{#each item.subItems as subItem}
											<Button.Root
												onclick={() => navigateTo(subItem.href)}
												class={`
                                                    w-full rounded-md px-3 py-2 text-left text-sm transition-colors
                                                    ${
																											isActive(subItem.href)
																												? 'bg-blue-50 text-blue-700 dark:bg-blue-900/20 dark:text-blue-300'
																												: 'text-gray-600 hover:bg-gray-100 hover:text-gray-900 dark:text-gray-400 dark:hover:bg-gray-700 dark:hover:text-gray-100'
																										}
                                                `}
											>
												{subItem.label}
											</Button.Root>
										{/each}
									</Collapsible.Content>
								{/if}
							</Collapsible.Root>
						{:else}
							<!-- Simple navigation item -->
							<Button.Root
								onclick={() => navigateTo(item.href)}
								class={`
                                    flex w-full items-center rounded-md px-3 py-2 text-sm transition-colors
                                    ${
																			isActive(item.href)
																				? 'border-r-2 border-blue-700 bg-blue-50 text-blue-700 dark:border-blue-400 dark:bg-blue-900/20 dark:text-blue-300'
																				: 'text-gray-700 hover:bg-gray-100 hover:text-gray-900 dark:text-gray-300 dark:hover:bg-gray-700 dark:hover:text-gray-100'
																		}
                                `}
							>
								<div class="flex-shrink-0">
									<ItemIcon class="h-5 w-5" />
								</div>
								{#if !isCollapsed}
									<span class="ml-3 font-medium">{item.label}</span>
								{/if}
							</Button.Root>
						{/if}
					</NavigationMenu.Item>
				{/each}
			</NavigationMenu.List>
		</NavigationMenu.Root>
	</div>

	<!-- Main content area -->
	<div class="flex min-w-0 flex-1 flex-col">
		<!-- Header bar for desktop and mobile -->
		<header
			class="flex h-16 items-center justify-between border-b border-gray-200 bg-white px-4 lg:px-6 dark:border-gray-700 dark:bg-gray-800"
		>
			<!-- Left section - Mobile menu button (mobile only) -->
			<div class="flex items-center">
				<Button.Root
					onclick={toggleMobileMenu}
					class="rounded-md p-2 text-gray-500 hover:bg-gray-100 hover:text-gray-900 lg:hidden dark:text-gray-400 dark:hover:bg-gray-700 dark:hover:text-gray-100"
				>
					<Menu class="h-5 w-5" />
				</Button.Root>

				<!-- Logo for mobile -->
				<div class="ml-2 lg:hidden">
					<span class="text-lg font-semibold text-gray-900 dark:text-gray-100"><Logo /></span>
				</div>
			</div>

			<!-- Right section - Actions -->
			<div class="flex items-center space-x-3">
				<!-- Dark mode toggle -->
				<Toggle.Root
					bind:pressed={isDarkMode}
					onPressedChange={toggleDarkMode}
					class="rounded-md p-2 text-gray-500 transition-colors hover:bg-gray-100 hover:text-gray-900 dark:text-gray-400 dark:hover:bg-gray-700 dark:hover:text-gray-100"
					aria-label="切换深色模式"
				>
					{#if isDarkMode}
						<Sun class="h-5 w-5" />
					{:else}
						<Moon class="h-5 w-5" />
					{/if}
				</Toggle.Root>

				<!-- Language dropdown -->
				<DropdownMenu.Root>
					<DropdownMenu.Trigger
						class="rounded-md p-2 text-gray-500 transition-colors hover:bg-gray-100 hover:text-gray-900 dark:text-gray-400 dark:hover:bg-gray-700 dark:hover:text-gray-100"
					>
						<Globe class="h-5 w-5" />
					</DropdownMenu.Trigger>
					<DropdownMenu.Portal>
						<DropdownMenu.Content
							class="z-50 min-w-[160px] rounded-lg border border-gray-200 bg-white py-1 shadow-lg dark:border-gray-700 dark:bg-gray-800"
							sideOffset={8}
						>
							{#each languages as lang}
								<DropdownMenu.Item
									onclick={() => changeLanguage(lang.code)}
									class="flex cursor-pointer items-center px-3 py-2 text-sm text-gray-700 hover:bg-gray-100 dark:text-gray-300 dark:hover:bg-gray-700 {currentLanguage ===
									lang.code
										? 'bg-blue-50 text-blue-700 dark:bg-blue-900/20 dark:text-blue-300'
										: ''}"
								>
									<span class="mr-2">{lang.flag}</span>
									<span>{lang.name}</span>
								</DropdownMenu.Item>
							{/each}
						</DropdownMenu.Content>
					</DropdownMenu.Portal>
				</DropdownMenu.Root>

				<!-- Notifications -->
				<Button.Root
					class="relative rounded-md p-2 text-gray-500 transition-colors hover:bg-gray-100 hover:text-gray-900 dark:text-gray-400 dark:hover:bg-gray-700 dark:hover:text-gray-100"
				>
					<Bell class="h-5 w-5" />
					<!-- Notification badge -->
					<span class="absolute -right-1 -top-1 h-3 w-3 rounded-full bg-red-500"></span>
				</Button.Root>

				<!-- User menu -->
				<DropdownMenu.Root>
					<DropdownMenu.Trigger
						class="rounded-md p-2 text-gray-500 transition-colors hover:bg-gray-100 hover:text-gray-900 dark:text-gray-400 dark:hover:bg-gray-700 dark:hover:text-gray-100"
					>
						<User class="h-5 w-5" />
					</DropdownMenu.Trigger>
					<DropdownMenu.Portal>
						<DropdownMenu.Content
							class="z-50 min-w-[200px] rounded-lg border border-gray-200 bg-white py-1 shadow-lg dark:border-gray-700 dark:bg-gray-800"
							sideOffset={8}
						>
							<DropdownMenu.Item
								class="flex cursor-pointer items-center px-3 py-2 text-sm text-gray-700 hover:bg-gray-100 dark:text-gray-300 dark:hover:bg-gray-700"
							>
								<User class="mr-2 h-4 w-4" />
								个人资料
							</DropdownMenu.Item>
							<DropdownMenu.Item
								class="flex cursor-pointer items-center px-3 py-2 text-sm text-gray-700 hover:bg-gray-100 dark:text-gray-300 dark:hover:bg-gray-700"
							>
								<Settings class="mr-2 h-4 w-4" />
								设置
							</DropdownMenu.Item>
							<DropdownMenu.Separator class="my-1 border-t border-gray-200 dark:border-gray-700" />
							<DropdownMenu.Item
								class="flex cursor-pointer items-center px-3 py-2 text-sm text-red-600 hover:bg-gray-100 dark:text-red-400 dark:hover:bg-gray-700"
							>
								退出登录
							</DropdownMenu.Item>
						</DropdownMenu.Content>
					</DropdownMenu.Portal>
				</DropdownMenu.Root>
			</div>
		</header>

		<!-- Page content -->
		<main class="flex-1 overflow-auto p-5">
			{@render children()}
		</main>
	</div>
</div>
