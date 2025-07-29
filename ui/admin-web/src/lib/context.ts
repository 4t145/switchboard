import type { ViewBox } from "./types/sbh-flow";

export interface SbhFlowViewBoxContext {
    getViewBox: () => ViewBox;
}

export const SBH_FLOW_VIEW_BOX_CONTEXT: unique symbol = Symbol("sbhFlowViewBox");