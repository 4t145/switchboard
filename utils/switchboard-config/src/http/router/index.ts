import { createRouterClass, createRouterConstructor } from "../index";
import { pathMatchRouterConfigSchema, hostRouterConfigSchema } from '../types'

export const createPathMatch = createRouterConstructor(createRouterClass("std.path-match", pathMatchRouterConfigSchema))
export const createHost = createRouterConstructor(createRouterClass("std.host", hostRouterConfigSchema))