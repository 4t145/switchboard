<script lang="ts">
	import { LayoutDashboard, Blocks, Database, Settings, ArrowRight } from 'lucide-svelte';
</script>

<div class="container mx-auto p-6 space-y-8">
	<!-- Header -->
	<div class="text-center space-y-2">
		<h1 class="h1">Welcome to Switchboard Controller</h1>
		<p class="text-surface-600 dark:text-surface-400">
			Manage your services, configurations, and system settings
		</p>
	</div>

	<!-- Quick navigation cards -->
	<div class="grid grid-cols-1 md:grid-cols-2 gap-6 max-w-4xl mx-auto">
		<!-- Dashboard Card -->
		<a
			href="/admin/dashboard"
			class="card p-6 hover:ring-2 hover:ring-primary-500 transition-all duration-200 space-y-3 group"
		>
			<div class="flex items-start justify-between">
				<div class="p-3 bg-primary-500/10 rounded-lg">
					<LayoutDashboard class="w-8 h-8 text-primary-500" />
				</div>
				<ArrowRight
					class="w-5 h-5 text-surface-400 group-hover:text-primary-500 group-hover:translate-x-1 transition-all"
				/>
			</div>
			<div>
				<h2 class="h3">Configuration Dashboard</h2>
				<p class="text-sm text-surface-600 dark:text-surface-400 mt-1">
					View and explore your current service configurations, bindings, and TLS settings
				</p>
			</div>
		</a>

		<!-- Services Builder Card -->
		<a
			href="/admin/build"
			class="card p-6 hover:ring-2 hover:ring-secondary-500 transition-all duration-200 space-y-3 group"
		>
			<div class="flex items-start justify-between">
				<div class="p-3 bg-secondary-500/10 rounded-lg">
					<Blocks class="w-8 h-8 text-secondary-500" />
				</div>
				<ArrowRight
					class="w-5 h-5 text-surface-400 group-hover:text-secondary-500 group-hover:translate-x-1 transition-all"
				/>
			</div>
			<div>
				<h2 class="h3">Service Builder</h2>
				<p class="text-sm text-surface-600 dark:text-surface-400 mt-1">
					Create and deploy new services with the interactive build wizard
				</p>
			</div>
		</a>

		<!-- Storage Card -->
		<a
			href="/admin/storage"
			class="card p-6 hover:ring-2 hover:ring-tertiary-500 transition-all duration-200 space-y-3 group"
		>
			<div class="flex items-start justify-between">
				<div class="p-3 bg-tertiary-500/10 rounded-lg">
					<Database class="w-8 h-8 text-tertiary-500" />
				</div>
				<ArrowRight
					class="w-5 h-5 text-surface-400 group-hover:text-tertiary-500 group-hover:translate-x-1 transition-all"
				/>
			</div>
			<div>
				<h2 class="h3">Storage Management</h2>
				<p class="text-sm text-surface-600 dark:text-surface-400 mt-1">
					Browse and manage stored objects, configurations, and data
				</p>
			</div>
		</a>

		<!-- Settings Card -->
		<a
			href="/admin/settings"
			class="card p-6 hover:ring-2 hover:ring-success-500 transition-all duration-200 space-y-3 group"
		>
			<div class="flex items-start justify-between">
				<div class="p-3 bg-success-500/10 rounded-lg">
					<Settings class="w-8 h-8 text-success-500" />
				</div>
				<ArrowRight
					class="w-5 h-5 text-surface-400 group-hover:text-success-500 group-hover:translate-x-1 transition-all"
				/>
			</div>
			<div>
				<h2 class="h3">Settings</h2>
				<p class="text-sm text-surface-600 dark:text-surface-400 mt-1">
					Customize your interface theme, language, and system preferences
				</p>
			</div>
		</a>
	</div>
</div>
