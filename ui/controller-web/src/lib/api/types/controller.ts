export type ControllerMeta = {
    version: string;
    build: string;
};

export type ControllerInfo = {
    name: string;
    description: string | null;
    meta: ControllerMeta;
};


export type StorageObjectDescriptor = {
    id: string;
    revision: string;
};

export type StorageObjectWithoutData = {
    descriptor: StorageObjectDescriptor;
    meta: StorageObjectMeta;
}
export type StorageObjectMeta = {
    created_at: Date;
    data_type: string;
}