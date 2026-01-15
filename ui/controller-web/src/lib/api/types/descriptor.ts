export type AnonServiceDescriptor = {
	provider: string;
	tls: string | null;
	config: string | null;
};

export type NamedServiceDescriptor = string;

// Serialized via Display/FromStr; use the string form when exchanging with the API
export type ServiceDescriptor = string;
