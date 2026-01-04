import type { ServiceConfig, KernelConnectionAndState, ResultObject } from "../types";
import { fetchJson } from "./index";
// 类型占位，可根据后端实际结构调整
export type KernelSummary = Record<string, KernelConnectionAndState>;
export type ConfigUpdateResults = Array<[string, ResultObject<null>]>;

export const kernelManagerApi = {
    listKernels: () => fetchJson<KernelSummary>('/api/kernel_manager/kernels'),
    updateConfig: (config: ServiceConfig) => fetchJson<ConfigUpdateResults>('/api/kernel_manager/update_config', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ config }),
    }),

};