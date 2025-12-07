export type ControllerMeta = {
    version: string;
    build: string;
};

export type ControllerInfo = {
    name: string;
    description: string | null;
    meta: ControllerMeta;
};
