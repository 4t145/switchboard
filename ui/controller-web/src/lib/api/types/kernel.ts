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
	| { kind: 'WaitingConfig' }
	| { kind: 'Running'; data: { config_version: string } }
	| {
			kind: 'Updating';
			data: {
				original_config_version: string;
				new_config_version: string;
			};
	  }
	| {
			kind: 'Preparing';
			data: {
				transaction_id: string;
				target_version: string;
			};
	  }
	| {
			kind: 'Prepared';
			data: {
				transaction_id: string;
				target_version: string;
			};
	  }
	| {
			kind: 'Committing';
			data: {
				transaction_id: string;
				target_version: string;
			};
	  }
	| { kind: 'ShuttingDown' }
	| { kind: 'Stopped' };

export type KernelState = {
	// Serialized as RFC3339 timestamp string
	since: string;
} & KernelStateKind;

export type KernelInfoAndState = {
	info: KernelInfo;
	state: KernelState;
};

export type KernelConnectionAndState =
	| { connection: 'Connected'; state: KernelInfoAndState }
	| { connection: 'Disconnected' };

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
