import type { Component } from 'svelte';

/**
 * 数据类型组件通用 Props 接口
 */
export type DataTypeComponentProps<T, P extends Record<string, unknown>> = {
	mode?: 'edit' | 'view';
	value?: T;
	disabled?: boolean;
	readonly?: boolean;
} & P;

/**
 * 数据类型元信息
 */
export type DataTypeMetadata<T, P extends Record<string, unknown>> = {
	/** 唯一类型标识符 */
	type: string;

	/** 显示名称 */
	displayName: string;

	/** 对应的 Svelte 组件（支持 edit/view 模式） */
	component: Component<DataTypeComponentProps<T, P>>;

	/** 默认值生成器 */
	defaultValue: () => T;

	/** 数据格式（用于 LinkOrValue 解析） */
	dataFormat: 'string' | 'object';
};

export type UnknownDataTypeMetadata = DataTypeMetadata<unknown, Record<string, unknown>>;
/**
 * 数据类型注册表实现
 */
class DataTypeRegistryImpl {
	private types = new Map<string, UnknownDataTypeMetadata>();
	/**
	 * 注册一个数据类型
	 */
	register<T, P extends Record<string, unknown>>(metadata: DataTypeMetadata<T, P>): void {
		if (this.types.has(metadata.type)) {
			console.warn(
				`[DataTypeRegistry] Type "${metadata.type}" is already registered, overwriting...`
			);
		}
		this.types.set(metadata.type, metadata as UnknownDataTypeMetadata);
	}

	/**
	 * 获取指定类型的元信息
	 */
	get(type: string): UnknownDataTypeMetadata | undefined {
		return this.types.get(type);
	}

	/**
	 * 获取所有已注册的类型元信息
	 */
	getAll(): UnknownDataTypeMetadata[] {
		return Array.from(this.types.values());
	}

	/**
	 * 检查类型是否已注册
	 */
	has(type: string): boolean {
		return this.types.has(type);
	}

	/**
	 * 获取所有已注册的类型标识符
	 */
	getAllTypes(): string[] {
		return Array.from(this.types.keys());
	}
}

/**
 * 全局数据类型注册表单例
 */
export const dataTypeRegistry = new DataTypeRegistryImpl();
