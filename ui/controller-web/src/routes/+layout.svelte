<script lang="ts">
	import '../app.css';
	import { registerDataTypes } from '$lib/data-types/config';

	let { children } = $props();

	// 应用启动时注册所有数据类型
	registerDataTypes();
</script>

{@render children()}
