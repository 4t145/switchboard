<script lang="ts">
	import { LayoutDashboard, Blocks, Database, Settings, ArrowRight } from '@lucide/svelte';
</script>

<div class="container mx-auto space-y-8 p-6">
	<!-- Header -->
	<div class="space-y-2 text-center">
		<h1 class="h1">Welcome to Switchboard Controller</h1>
		<p class="text-surface-600 dark:text-surface-400">
			Manage your services, configurations, and system settings
		</p>
	</div>

	<!-- Quick navigation cards -->
	<div class="mx-auto grid max-w-4xl grid-cols-1 gap-6 md:grid-cols-2">
		<!-- Dashboard Card -->
		<a
			href="/admin/dashboard"
			class="group space-y-3 card p-6 transition-all duration-200 hover:ring-2 hover:ring-primary-500"
		>
			<div class="flex items-start justify-between">
				<div class="rounded-lg bg-primary-500/10 p-3">
					<LayoutDashboard class="h-8 w-8 text-primary-500" />
				</div>
				<ArrowRight
					class="h-5 w-5 text-surface-400 transition-all group-hover:translate-x-1 group-hover:text-primary-500"
				/>
			</div>
			<div>
				<h2 class="h3">Configuration Dashboard</h2>
				<p class="mt-1 text-sm text-surface-600 dark:text-surface-400">
					View and explore your current service configurations, bindings, and TLS settings
				</p>
			</div>
		</a>

		<!-- Services Builder Card -->
		<a
			href="/admin/build"
			class="group space-y-3 card p-6 transition-all duration-200 hover:ring-2 hover:ring-secondary-500"
		>
			<div class="flex items-start justify-between">
				<div class="rounded-lg bg-secondary-500/10 p-3">
					<Blocks class="h-8 w-8 text-secondary-500" />
				</div>
				<ArrowRight
					class="h-5 w-5 text-surface-400 transition-all group-hover:translate-x-1 group-hover:text-secondary-500"
				/>
			</div>
			<div>
				<h2 class="h3">Service Builder</h2>
				<p class="mt-1 text-sm text-surface-600 dark:text-surface-400">
					Create and deploy new services with the interactive build wizard
				</p>
			</div>
		</a>

		<!-- Storage Card -->
		<a
			href="/admin/storage"
			class="group space-y-3 card p-6 transition-all duration-200 hover:ring-2 hover:ring-tertiary-500"
		>
			<div class="flex items-start justify-between">
				<div class="rounded-lg bg-tertiary-500/10 p-3">
					<Database class="h-8 w-8 text-tertiary-500" />
				</div>
				<ArrowRight
					class="h-5 w-5 text-surface-400 transition-all group-hover:translate-x-1 group-hover:text-tertiary-500"
				/>
			</div>
			<div>
				<h2 class="h3">Storage Management</h2>
				<p class="mt-1 text-sm text-surface-600 dark:text-surface-400">
					Browse and manage stored objects, configurations, and data
				</p>
			</div>
		</a>

		<!-- Settings Card -->
		<a
			href="/admin/settings"
			class="group space-y-3 card p-6 transition-all duration-200 hover:ring-2 hover:ring-success-500"
		>
			<div class="flex items-start justify-between">
				<div class="rounded-lg bg-success-500/10 p-3">
					<Settings class="h-8 w-8 text-success-500" />
				</div>
				<ArrowRight
					class="h-5 w-5 text-surface-400 transition-all group-hover:translate-x-1 group-hover:text-success-500"
				/>
			</div>
			<div>
				<h2 class="h3">Settings</h2>
				<p class="mt-1 text-sm text-surface-600 dark:text-surface-400">
					Customize your interface theme, language, and system preferences
				</p>
			</div>
		</a>
	</div>
</div>
