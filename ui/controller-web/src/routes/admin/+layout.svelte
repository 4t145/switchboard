<script lang="ts">
	import { Navigation } from '@skeletonlabs/skeleton-svelte';
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
		User,
		ArrowLeftRightIcon,
		Factory
	} from 'lucide-svelte';
	import Logo from '$lib/components/logo.svelte';
	let { children } = $props();

	let isLayoutRail = $state(false);
	let isMobileMenuOpen = $state(false);
	let isDarkMode = $state(false);
	let showLangMenu = $state(false);
	let showUserMenu = $state(false);
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
		user: User,
		factory: Factory
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
			href: '/admin/services'
		},
		{ label: 'Build', icon: 'factory', href: '/admin/build' },
		{ label: 'Storage', icon: 'database', href: '/admin/storage' }
	];

	// 计算当前活跃路径
	let currentPath = $derived(page.url.pathname);

	function toggleLayoutRail() {
		isLayoutRail = !isLayoutRail;
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
	<!-- Sidebar -->
	<div
		class={`
        fixed inset-y-0 left-0 z-50 transform 
        transition-all duration-300 ease-in-out lg:relative dark:border-gray-700 dark:bg-gray-800
    `}
	>
		<!-- Navigation using Skeleton Navigation -->
		<Navigation
			layout={isLayoutRail ? 'rail' : 'sidebar'}
			class={isLayoutRail ? '' : 'grid grid-rows-[1fr_auto] gap-4'}
		>
			<Navigation.Header>
				<Navigation.Trigger onclick={toggleLayoutRail}>
					<ArrowLeftRightIcon class={isLayoutRail ? 'size-5' : 'size-4'} />
					{#if !isLayoutRail}<span>Resize</span>{/if}
				</Navigation.Trigger>
			</Navigation.Header>
			<Navigation.Content>
				<!-- 主导航（无分组项） -->
				<Navigation.Group>
					<Navigation.Label class={`pl-2 ${isLayoutRail ? 'sr-only' : ''}`}>Main</Navigation.Label>
					<Navigation.Menu>
						{#each navigationItems.filter((i) => !i.subItems) as link (link.href)}
							{@const Icon = iconComponents[link.icon]}
							<Navigation.TriggerAnchor
								href={link.href}
								onclick={() => {
									navigateTo(link.href);
								}}
								aria-current={isActive(link.href) ? 'page' : undefined}
							>
								<Icon class="size-4" />
								<Navigation.TriggerText>{link.label}</Navigation.TriggerText>
							</Navigation.TriggerAnchor>
						{/each}
					</Navigation.Menu>
				</Navigation.Group>
			</Navigation.Content>
			<Navigation.Footer>
				<Navigation.TriggerAnchor
					href="/admin/settings"
					title="Settings"
					aria-label="Settings"
					class="rounded px-2 py-2 hover:bg-gray-50 dark:hover:bg-gray-700/50"
				>
					<Settings class="size-4" />
					{#if !isLayoutRail}
						<Navigation.TriggerText>Settings</Navigation.TriggerText>
					{/if}
				</Navigation.TriggerAnchor>
			</Navigation.Footer>
		</Navigation>
	</div>

	<!-- Main content area -->
	<div class="flex min-w-0 flex-1 flex-col">
		<!-- Page content -->
		<main class="flex-1 overflow-auto">
			{@render children()}
		</main>
	</div>
</div>
