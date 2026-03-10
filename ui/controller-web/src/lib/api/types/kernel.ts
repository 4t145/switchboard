export type KernelMeta = {
	version: string;
	build: string;
};

export type KernelInfo = {
	name: string;
	id: string;
	description: string | null;
	meta: KernelMeta;
};

export type KernelStateKind =
	| { kind: 'waiting_config' }
	| { kind: 'running'; data: { config_version: string } }
	| {
			kind: 'updating';
			data: {
				original_config_version: string;
				new_config_version: string;
			};
	  }
	| {
			kind: 'preparing';
			data: {
				transaction_id: string;
				target_version: string;
			};
	  }
	| {
			kind: 'prepared';
			data: {
				transaction_id: string;
				target_version: string;
			};
	  }
	| {
			kind: 'committing';
			data: {
				transaction_id: string;
				target_version: string;
			};
	  }
	| { kind: 'shutting_down' }
	| { kind: 'stopped' };

export type KernelState = {
	// Serialized as RFC3339 timestamp string
	since: string;
} & KernelStateKind;

export type KernelInfoAndState = {
	info: KernelInfo;
	state: KernelState;
};

export type KernelConnectionAndState =
	| { connection: 'connected'; state: KernelInfoAndState }
	| { connection: 'disconnected' };

export type RolloutPhase = 'prepare' | 'commit' | 'rollback_prepare' | 'rollback_commit';

export type RolloutStatus =
	| { status: 'succeeded' }
	| {
			status: 'failed';
			phase: RolloutPhase;
	  };

export type KernelRolloutResult = [string, import('./error').ResultObject<null>];

export type ConfigRolloutReport = {
	transaction_id: string;
	all_or_nothing: boolean;
	status: RolloutStatus;
	prepare_results: KernelRolloutResult[];
	commit_results: KernelRolloutResult[];
	abort_results: KernelRolloutResult[];
	rollback_transaction_id: string | null;
	rollback_prepare_results: KernelRolloutResult[];
	rollback_commit_results: KernelRolloutResult[];
	rollback_abort_results: KernelRolloutResult[];
};
