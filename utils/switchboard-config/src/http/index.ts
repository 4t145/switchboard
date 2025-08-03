// import { type ZodType, type z.infer } from "json-schema-to-ts"
import { z, ZodType } from "zod/v3"
const KIND: unique symbol = Symbol("kind");
export type PortKey = string;
export type PortKeys = PortKey[];
export type Kind<K extends `${string}`> = {
  kind: K;
};
export type Identifier = {
  id: string,
  name?: string;
}
export type IdentifierLike = Identifier | string | (() => Identifier);
export function intoIdentifier(id: IdentifierLike): Identifier {
  if (typeof id === "string") {
    return { id };
  }
  if (typeof id === "function") {
    return id();
  }
  return id.name ? { id: id.id, name: id.name } : { id: id.id };
}
export type DefaultPort = "$default";
export type Class<K extends `${string}` = `${string}`, C extends ZodType = ZodType> = {
  id: Identifier;
  configSchema: C;
} & Kind<K>;

export type Interface<I extends `${string}` = `${string}`, O extends `${string}` = `${string}`> = {
  input: Inputs<I>;
  output: Outputs<O>;
}
export type Instance<C extends Class = Class, I extends Interface = Interface> = {
  // [KIND]: "Instance";
  id: Identifier;
  class: C["id"]["id"];
  config: z.infer<C["configSchema"]>;
  interface: I
};
export type Inputs<P extends string = `${string}`> = Record<P, void>;
export const DEFAULT_INPUT: Inputs = { _: void 0 };
export type Outputs<P extends string = `${string}`> = Record<P, TargetLike>;
export type Target<P extends PortKey = PortKey> = {
  id: string;
  port: P;
}
export type GetDefaultTarget = {
  getDefaultTarget(): Target;
}
export type GetPort<I> = {
  getPort(port: I): Target;
}
export type TargetLike =
  | GetDefaultTarget
  | Target;
export function getTarget(target: TargetLike): Target {
  if ("getDefaultTarget" in target) {
    return target.getDefaultTarget();
  }
  return target;
}
export type RouterClass<Z extends ZodType = ZodType> = Class<"Router", Z>;
export type RouterInstance<Output extends `${string}` = `${string}`, C extends RouterClass = RouterClass> = Instance<C, Interface<DefaultPort, Output>> & GetDefaultTarget;

export type LayerClass<Z extends ZodType = ZodType> = Class<"Layer", Z>;
export type LayerInstance<C extends LayerClass = LayerClass> = Instance<C, Interface<DefaultPort, DefaultPort>> & GetDefaultTarget;

export type ServiceClass<Z extends ZodType = ZodType> = Class<"Service", Z>;
export type ServiceInstance<C extends ServiceClass = ServiceClass> = Instance<C, Interface<DefaultPort, never>> & GetDefaultTarget;

export type BundleClass<Z extends ZodType = ZodType> = Class<"Bundle", Z>;
export type BundleInstance<I extends `${string}` = `${string}`, O extends `${string}` = `${string}`, C extends BundleClass = BundleClass> = Instance<C, Interface<I, O>> & GetPort<I>;
export function Class<K extends `${string}`, Z extends ZodType>(kind: K, id: IdentifierLike, configSchema: Z): Class<K, Z> {
  return {
    id: intoIdentifier(id),
    configSchema,
    kind,
  };
}
export const RouterClass = <Z extends ZodType>(id: IdentifierLike, configSchema: Z): RouterClass => Class("Router", id, configSchema);
export const LayerClass = <Z extends ZodType>(id: IdentifierLike, configSchema: Z): LayerClass => Class("Layer", id, configSchema);
export const ServiceClass = <Z extends ZodType>(id: IdentifierLike, configSchema: Z): ServiceClass<Z> => Class("Service", id, configSchema);
export const BundleClass = <Z extends ZodType>(id: IdentifierLike, configSchema: Z): BundleClass<Z> => Class("Bundle", id, configSchema);
export function createClass<K extends "Router" | "Layer" | "Service" | "Bundle", Z extends ZodType>(kind: K, id: IdentifierLike, configSchema: Z): Class<K, Z> {
  return {
    id: intoIdentifier(id),
    configSchema,
    kind,
  };
}
export function createRouterClass<Z extends ZodType>(id: IdentifierLike, configSchema: Z): RouterClass<Z> {
  return createClass("Router", id, configSchema);
}
export function createLayerClass<Z extends ZodType>(id: IdentifierLike, configSchema: Z): LayerClass {
  return createClass("Layer", id, configSchema);
}
export function createServiceClass<Z extends ZodType>(id: IdentifierLike, configSchema: Z): ServiceClass {
  return createClass("Service", id, configSchema);
}
export function createBundleClass<Z extends ZodType>(id: IdentifierLike, configSchema: Z): BundleClass {
  return createClass("Bundle", id, configSchema);
}
export function getDefaultTarget(id: string): Target {
  return {
    id,
    port: "_"
  };
}

export function createRouterConstructor<C extends RouterClass, P extends Outputs>(cls: C):
  (id: IdentifierLike, ports: P, config: z.infer<C['configSchema']>) => RouterInstance<Extract<keyof P, string>, C> {
  return (id: IdentifierLike, ports: P, config: z.infer<C['configSchema']>) => {
    const ident = intoIdentifier(id);
    return {
      id: ident,
      class: cls.id.id,
      config,
      interface: {
        input: { $default: void 0 },
        output: ports
      },
      getDefaultTarget: () => getDefaultTarget(ident.id),
    };
  };
}

export function createLayerConstructor<C extends LayerClass>(cls: C): (id: IdentifierLike, next: TargetLike, config: any) => LayerInstance<C> {
  return (id: IdentifierLike, next: TargetLike, config: z.infer<C['configSchema']>) => {
    const ident = intoIdentifier(id);
    return {
      id: ident,
      class: cls.id.id,
      config,
      interface: { input: DEFAULT_INPUT, output: { $default: next } },
      getDefaultTarget: () => getDefaultTarget(ident.id),
    };
  };
}

export function createServiceConstructor<C extends ServiceClass>(cls: C): (id: IdentifierLike, config: any) => ServiceInstance<C> {
  return (id: IdentifierLike, config: z.infer<C['configSchema']>) => {
    const ident = intoIdentifier(id);
    return {
      id: ident,
      class: cls.id.id,
      config,
      interface: { input: DEFAULT_INPUT, output: {} },
      getDefaultTarget: () => getDefaultTarget(ident.id),
    };
  };
}

export function createBundleConstructor<I extends `${string}`, O extends `${string}`, C extends BundleClass>(cls: C):
  (id: IdentifierLike, inputs: Inputs<I>, outputs: Outputs<O>, config: z.infer<C['configSchema']>) => BundleInstance<I, O, C> {
  return (id: IdentifierLike, input: Inputs<I>, output: Outputs<O>, config: z.infer<C['configSchema']>) => {
    const ident = intoIdentifier(id);
    return {
      id: ident,
      class: cls.id.id,
      config,
      interface: { input, output },
      getPort: (port: I) => {
        if (port in input) {
          return {
            id: ident.id,
            port,
          };
        }
        throw new Error(`Port ${port} not found in outputs`);
      },
    };
  };
}

export type Incoming = {
  incoming: TargetLike;
}

export type ValidInstance = RouterInstance | LayerInstance | ServiceInstance | BundleInstance;
export type Config = {
  incoming: Target;
  instances: Instance[]
}

export function createConfig(incoming: TargetLike, instances: Instance[]): Config {
  return {
    incoming: getTarget(incoming),
    instances,
  };
}
// import { createHost, createPathMatch } from "./router";
// const host = createHost("host", { $default: { id: "default", port: "default" } }, {});
// const pathMatch = createPathMatch("path-match", { $default: { id: "default", port: "default" } }, []);
// const config = createConfig(host, [host, pathMatch])