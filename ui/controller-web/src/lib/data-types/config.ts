import { dataTypeRegistry } from './registry';

// 导入数据类型组件
import JsonEditor from './components/json-editor.svelte';
import PemEditor from './components/pem-editor.svelte';
import ProviderConfig from './components/provider-config.svelte';
import HttpClassConfig from './components/http-class-config.svelte';

/**
 * 注册所有数据类型到全局注册表
 * 此函数应在应用启动时调用一次
 */
export function registerDataTypes() {
	console.log('[DataTypes] Registering data types...');

	// PEM 相关类型
	dataTypeRegistry.register({
		type: 'pem',
		displayName: 'PEM Certificate/Key',
		component: PemEditor,
		defaultValue: () => '',
		dataFormat: 'string'
	});

	dataTypeRegistry.register({
		type: 'PemFile',
		displayName: 'PEM File',
		component: PemEditor,
		defaultValue: () => '',
		dataFormat: 'string'
	});

	dataTypeRegistry.register({
		type: 'PemsFile',
		displayName: 'Multiple PEM Files',
		component: PemEditor,
		defaultValue: () => '',
		dataFormat: 'string'
	});

	// 配置类型（使用 Provider 插件系统）
	dataTypeRegistry.register({
		type: 'ServiceConfig',
		displayName: 'Service Configuration',
		component: ProviderConfig,
		defaultValue: () => ({ provider: '', config: {} }),
		dataFormat: 'object'
	});

	dataTypeRegistry.register({
		type: 'TcpServiceConfig',
		displayName: 'TCP Service Configuration',
		component: ProviderConfig,
		defaultValue: () => ({}),
		dataFormat: 'object'
	});

	// HTTP 类配置类型（使用 HTTP 类插件系统）
	dataTypeRegistry.register({
		type: 'HttpClassConfig',
		displayName: 'HTTP Class Configuration',
		component: HttpClassConfig,
		defaultValue: () => ({}),
		dataFormat: 'object'
	});

	// 通用 JSON 类型（降级方案）
	dataTypeRegistry.register({
		type: 'json',
		displayName: 'JSON Object',
		component: JsonEditor,
		defaultValue: () => ({}),
		dataFormat: 'object'
	});

	const registeredTypes = dataTypeRegistry.getAllTypes();
	console.log(`[DataTypes] Successfully registered ${registeredTypes.length} types:`, registeredTypes);
}
