// import { type ZodType, type z.infer } from "json-schema-to-ts"
import { z } from "zod/v3";
import { symbol } from "zod/v4";
import type { NamedService } from "..";

const KIND: unique symbol = Symbol("kind");
export const DEFAULT_PORT = "$default" as const;
export type InstanceId = string;

export type FilterId = InstanceId;

export type NodeId = InstanceId;

export type NodePort = string;

export type ClassId = {
  namespace?: string;
  name: string;
};

export type ClassMeta = {
  version: string;
  description?: string;
  author?: string;
  license?: string;
  repository?: string;
  homepage?: string;
};

export type NodeTarget = {
  id: NodeId;
  port: NodePort;
};

export type FlowOptions = {
  maxLoop?: number;
};

export type InstanceType = "Node" | "Filter";
export type ClassData<
  C extends ClassId = ClassId,
  T extends InstanceType = InstanceType,
  S extends z.ZodType = z.ZodType,
> = {
  id: ClassId;
  meta: ClassMeta;
  instanceType: T;
  configSchema: S;
  createInstance: CreateInstance<C, T, S>;
};
export type CreateInstance<
  C extends ClassId = ClassId,
  T extends InstanceType = InstanceType,
  S extends z.ZodType = z.ZodType,
> = {
  (id: string, name: string, config: z.infer<S>): Instance<C, T, S>;
  (id: string, config: z.infer<S>): Instance<C, T, S>;
};
export function declareClass<
  C extends ClassId,
  T extends InstanceType,
  S extends z.ZodType,
>(
  classId: C,
  instanceType: T,
  configSchema: S,
  meta: ClassMeta
): ClassData<C, T, S> {
  const createInstance: CreateInstance<C, T, S> = (
    ...args: [string, string, z.infer<S>] | [string, z.infer<S>]
  ) => {
    if (args.length === 3) {
      const [name, id, config] = args;
      const data = {
        name,
        class: classId,
        type: instanceType,
        config,
      };
      return {
        id,
        data,
        [TARGET_LIKE_MARKER]: () => ({
          id,
          port: DEFAULT_PORT,
        }),
      };
    } else {
      const [config, id] = args;
      const data = {
        class: classId,
        type: instanceType,
        config,
      };
      return {
        id,
        data,
        [TARGET_LIKE_MARKER]: () => ({
          id,
          port: DEFAULT_PORT,
        }),
      };
    }
  };
  return {
    id: classId,
    meta,
    instanceType,
    configSchema,
    createInstance,
  };
}

export type Instance<
  C extends ClassId = ClassId,
  T extends InstanceType = InstanceType,
  S extends z.ZodType = z.ZodType,
> = {
  id: InstanceId;
  data: InstanceData<C, T, S>;
} & TargetLikeInterface;

export type InstanceData<
  C extends ClassId = ClassId,
  T extends InstanceType = InstanceType,
  S extends z.ZodType = z.ZodType,
> = {
  name?: string;
  class: C;
  type: T;
  config: z.infer<S>;
};

export type FlowConfig = {
  entrypoint: NodeTarget;
  instances: Record<InstanceId, InstanceData>;
  options: FlowOptions;
};

export const TARGET_LIKE_MARKER: unique symbol = Symbol("target-like");
export type TargetLikeInterface = {
  [TARGET_LIKE_MARKER](): NodeTarget;
};
export type TargetLike = TargetLikeInterface | NodeTarget;
export function isTargetLike(value: unknown): value is TargetLike {
  return (
    typeof value === "object" && value !== null && TARGET_LIKE_MARKER in value
  );
}
export function asTarget(target: TargetLike): NodeTarget {
  if (TARGET_LIKE_MARKER in target) {
    return target[TARGET_LIKE_MARKER]();
  } else {
    return target;
  }
}
export type HttpConfigBuilder = {
  instances: Record<InstanceId, InstanceData>;
  addInstance: <C extends ClassId, T extends InstanceType, S extends z.ZodType>(
    instance: Instance<C, T, S>
  ) => HttpConfigBuilder;
  build(entrypoint: TargetLike, options?: FlowOptions): FlowConfig;
};

export function resolveAllTargets(value: unknown): NodeTarget | unknown {
  if (isTargetLike(value)) {
    return asTarget(value);
  } else if (value instanceof Array) {
    return value.map(resolveAllTargets);
  } else if (typeof value === "object" && value !== null) {
    return Object.fromEntries(
      Object.entries(value).map(([key, val]) => [key, resolveAllTargets(val)])
    );
  } else {
    return value;
  }
}

export function createHttpConfig(): HttpConfigBuilder {
  const flow: HttpConfigBuilder = {
    instances: {},
    addInstance: (instance) => {
      instance.data.config = resolveAllTargets(instance.data.config);
      flow.instances[instance.id] = instance.data;
      return flow;
    },
    build: (entrypoint, options) => {
      return {
        entrypoint: asTarget(entrypoint),
        instances: flow.instances,
        options: options ?? {},
      };
    },
  };
  return flow;
}

export * as Router from "./router/index";
export function createHttpNamedService(
  name: string,
  flowConfig: FlowConfig,
  options?: {
    config?: string;
    description?: string;
    tls?: string;
  }
): NamedService {
  const config = JSON.stringify(flowConfig, null, 2);
  return {
    provider: "http",
    name,
    config,
    ...options,
  };
}
