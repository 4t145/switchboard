<script lang="ts">
	import { Navigation } from '@skeletonlabs/skeleton-svelte';
	import { page } from '$app/state';
	import { goto } from '$app/navigation';
	import { onMount } from 'svelte';
	import {
		Menu,
		Home,
		Activity,
		Database,
		Settings,
		Factory,
		X
	} from 'lucide-svelte';
	import Logo from '$lib/components/logo.svelte';
	import { settingsStore } from '$lib/stores/settings.svelte';
	
	let { children } = $props();

	let isLayoutRail = $state(false);
	let isMobileMenuOpen = $state(false);

	// Initialize settings on mount
	onMount(() => {
		settingsStore.load();
		settingsStore.apply();

		// Setup responsive rail/sidebar detection
		const xlMediaQuery = window.matchMedia('(min-width: 1280px)'); // xl breakpoint
		
		// Initial check
		const setLayout = () => {
			// xl and above: sidebar (wide enough for full sidebar)
			// lg to xl: rail (medium screen, compact mode)
			// below lg: hidden (mobile mode)
			if (window.innerWidth >= 1280) {
				isLayoutRail = false; // Full sidebar on wide screens
			} else if (window.innerWidth >= 1024) {
				isLayoutRail = true; // Rail mode on medium screens
			}
			// Below lg: mobile mode handled by CSS
		};

		setLayout(); // Initial setup

		// Listen for changes
		const handleResize = () => setLayout();
		
		xlMediaQuery.addEventListener('change', handleResize);
		window.addEventListener('resize', handleResize);

		return () => {
			xlMediaQuery.removeEventListener('change', handleResize);
			window.removeEventListener('resize', handleResize);
		};
	});

	// 图标组件映射
	const iconComponents = {
		menu: Menu,
		home: Home,
		activity: Activity,
		database: Database,
		settings: Settings,
		factory: Factory,
		x: X
	};

	const navigationItems: {
		label: string;
		icon: keyof typeof iconComponents;
		href: string;
		subItems?: { label: string; href: string }[];
	}[] = [
		{ label: 'Dashboard', icon: 'home', href: '/admin/dashboard' },
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

	function toggleMobileMenu() {
		isMobileMenuOpen = !isMobileMenuOpen;
	}

	function navigateTo(href: string) {
		goto(href);
		isMobileMenuOpen = false;
	}

	function isActive(href: string): boolean {
		// Exact match for specific routes
		return currentPath === href || currentPath.startsWith(href + '/');
	}
</script>

<style>
	/* Active navigation item styling */
	:global([aria-current="page"]) {
		background-color: rgb(var(--color-primary-100) / 1);
		color: rgb(var(--color-primary-700) / 1);
	}

	:global(.dark [aria-current="page"]) {
		background-color: rgb(var(--color-primary-900) / 0.3);
		color: rgb(var(--color-primary-400) / 1);
	}

	:global([aria-current="page"] svg) {
		color: rgb(var(--color-primary-600) / 1);
	}

	:global(.dark [aria-current="page"] svg) {
		color: rgb(var(--color-primary-400) / 1);
	}

	/* Active indicator bar */
	:global([aria-current="page"]::before) {
		content: '';
		position: absolute;
		left: 0;
		top: 50%;
		height: 2rem;
		width: 0.25rem;
		transform: translateY(-50%);
		border-radius: 0 9999px 9999px 0;
		background-color: rgb(var(--color-primary-600) / 1);
	}

	:global(.dark [aria-current="page"]::before) {
		background-color: rgb(var(--color-primary-400) / 1);
	}

	/* Navigation item base styles */
	:global(.navigation-item) {
		position: relative;
		display: flex;
		align-items: center;
		gap: 0.75rem;
		border-radius: 0.5rem;
		padding: 0.625rem 0.75rem;
		font-size: 0.875rem;
		font-weight: 500;
		color: rgb(var(--color-surface-700) / 1);
		transition: all 150ms;
	}

	:global(.navigation-item:hover) {
		background-color: rgb(var(--color-surface-100) / 1);
	}

	:global(.dark .navigation-item) {
		color: rgb(var(--color-surface-300) / 1);
	}

	:global(.dark .navigation-item:hover) {
		background-color: rgb(var(--color-surface-700) / 0.5);
	}
</style>

<!-- Mobile Header (visible only on small screens) -->
<div class="sticky top-0 z-40 flex items-center justify-between border-b border-surface-200 bg-white px-4 py-3 dark:border-surface-700 dark:bg-surface-900 lg:hidden">
	<div class="flex items-center gap-3">
		<Logo class="h-8 w-8" mode="overlap" />
		<span class="text-lg font-semibold text-surface-900 dark:text-surface-100">Switchboard</span>
	</div>
	
	<button
		onclick={toggleMobileMenu}
		class="rounded-lg p-2 hover:bg-surface-100 dark:hover:bg-surface-800"
		aria-label="Toggle menu"
	>
		{#if isMobileMenuOpen}
			<X class="size-6" />
		{:else}
			<Menu class="size-6" />
		{/if}
	</button>
</div>

<div class="flex min-h-screen bg-surface-50 dark:bg-surface-900">
	<!-- Mobile Overlay -->
	{#if isMobileMenuOpen}
		<div
			class="fixed inset-0 z-40 bg-black/50 lg:hidden"
			onclick={toggleMobileMenu}
			role="button"
			tabindex="-1"
			aria-label="Close menu"
		></div>
	{/if}

	<!-- Sidebar -->
	<aside
		class={`
			fixed inset-y-0 left-0 z-50 border-r border-surface-200 bg-white transition-all duration-300 ease-in-out dark:border-surface-700 dark:bg-surface-800
			lg:sticky lg:top-0 lg:h-screen lg:translate-x-0
			${isMobileMenuOpen ? 'translate-x-0' : '-translate-x-full'}
		`}
	>
		<!-- Navigation using Skeleton Navigation -->
		<Navigation layout={isLayoutRail ? 'rail' : 'sidebar'}>
			<div class="flex h-full flex-col">
				<!-- Header Section -->
				<Navigation.Header>
					<!-- Logo area (desktop only) -->
					<div class="mb-4 hidden px-4 pt-4 lg:block">
						<div class="flex items-center gap-3">
							<Logo class="h-8 w-8" mode="overlap" />
							{#if !isLayoutRail}
								<span class="text-lg font-semibold text-surface-900 dark:text-surface-100">Switchboard</span>
							{/if}
						</div>
					</div>
				</Navigation.Header>
				
				<!-- Content Section (flexible, takes remaining space) -->
				<div class="flex-1 overflow-y-auto">
					<Navigation.Content>
						<!-- 主导航（无分组项） -->
						<Navigation.Group>
							<Navigation.Label class={`pl-2 ${isLayoutRail ? 'sr-only' : ''}`}>Main</Navigation.Label>
							<Navigation.Menu>
								{#each navigationItems.filter((i) => !i.subItems) as link (link.href)}
									{@const Icon = iconComponents[link.icon]}
									{@const active = isActive(link.href)}
									<li>
										<a
											href={link.href}
											onclick={(e) => {
												e.preventDefault();
												navigateTo(link.href);
											}}
											aria-current={active ? 'page' : undefined}
											class="navigation-item"
											title={isLayoutRail ? link.label : undefined}
										>
											<Icon class="size-5" />
											<span class={isLayoutRail ? 'sr-only' : ''}>{link.label}</span>
										</a>
									</li>
								{/each}
							</Navigation.Menu>
						</Navigation.Group>
					</Navigation.Content>
				</div>
				
				<!-- Footer Section -->
				<Navigation.Footer>
					{@const settingsActive = isActive('/admin/settings')}
					<a
						href="/admin/settings"
						aria-current={settingsActive ? 'page' : undefined}
						class="navigation-item"
						title={isLayoutRail ? 'Settings' : undefined}
						onclick={(e) => {
							e.preventDefault();
							navigateTo('/admin/settings');
						}}
					>
						<Settings class="size-5" />
						{#if !isLayoutRail}
							<span>Settings</span>
						{/if}
					</a>
				</Navigation.Footer>
			</div>
		</Navigation>
	</aside>

	<!-- Main content area -->
	<div class="flex min-w-0 flex-1 flex-col">
		<!-- Page content -->
		<main class="flex-1 overflow-auto">
			{@render children()}
		</main>
	</div>
</div>
