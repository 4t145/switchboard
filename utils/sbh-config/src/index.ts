import { type JSONSchema, type FromSchema } from "json-schema-to-ts";
const KIND: unique symbol = Symbol("kind");
export type PortKey = string;
export type PortKeys = PortKey[];
export type Class = {
  // [KIND]: "Class";
  class: string;
  configSchema: JSONSchema;
};

export type Instance<C extends Class> = {
  // [KIND]: "Instance";
  name: string;
  id: string;
  class: C["class"];
  config: FromSchema<C["configSchema"]>;
  portKeys: PortKeys;
  getId(): string;
};
export type Ports<P extends string> = {
  [K in P]: TargetLike;
};
export type Reference<I extends Instance<Class>> = {
  reference: string;
  ports: Ports<I["portKeys"][number]>;
  getId(): string;
};

export type TargetLike =
  | {
      getId(): string;
    }
  | string;

export const TargetLike = {
  getId(T: TargetLike): string | null {
    if (typeof T === "string") {
      return T;
    } else if (typeof T.getId === "function") {
      return T.getId();
    } else {
      throw new Error(
        "Invalid TargetLike: must be a string or have a getId method"
      );
    }
  },
};

export type Kind<K extends `${string}`> = {
  $kind: K;
};
export type Router<C extends Class> = Kind<"Router"> & C;
export type Layer<C extends Class> = Kind<"Layer"> & C;
export type Service<C extends Class> = Kind<"Service"> & C;
export type Bundle<C extends Class> = Kind<"Bundle"> & C;

export type RouterInstance<C extends Class> = Kind<"Router"> & Instance<C> & {};
export type LayerInstance<C extends Class> = Kind<"Layer"> & Instance<C>;
export type ServiceInstance<C extends Class> = Kind<"Service"> & Instance<C>;
export type BundleInstance<C extends Class> = Kind<"Bundle"> & Instance<C>;

// 1. class
// 2. instance of class, have certain ports
// 3. reference of instance, can be used to connect to other instances
export const Url = {
  class: "Url",
  configSchema: {
    type: "object",
    properties: {
      path: { type: "string" },
      method: { type: "string", enum: ["GET", "POST", "PUT", "DELETE"] },
    },
    required: ["path", "method"],
  },
};
