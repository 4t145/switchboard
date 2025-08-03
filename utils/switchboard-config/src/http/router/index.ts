import { createRouterClass, routerConstructor } from "../index";
import { pathMatchRouterConfigSchema, hostRouterConfigSchema } from '../types'

export const pathMatch = routerConstructor(createRouterClass("std.path-match", pathMatchRouterConfigSchema))
export const host = routerConstructor(createRouterClass("std.host", hostRouterConfigSchema))