// import { createRouterClass, createRouterConstructor } from "../index";
import { declareClass } from "..";
import { pathMatchRouterConfigSchema, hostRouterConfigSchema } from "../types";
export const PathMatch = declareClass(
  {
    name: "path-match",
    namespace: "std",
  },
  "Node",
  pathMatchRouterConfigSchema,
  {
    version: "1.0.0",
  }
);

export const Host = declareClass(
  {
    name: "host",
    namespace: "std",
  },
  "Node",
  hostRouterConfigSchema,
  {
    version: "1.0.0",
  }
);

