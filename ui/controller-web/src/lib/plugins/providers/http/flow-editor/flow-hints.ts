export type HintType =
	| 'empty-state'
	| 'unconfigured'
	| 'cycle-detected'
	| 'connection-needed'
	| 'shortcut-help'
	| 'orphan-nodes'
	| 'entrypoint-not-set';

export type Hint = {
	id: string;
	type: HintType;
	severity: 'info' | 'warning' | 'error';
	title: string;
	message: string;
	actions?: {
		label: string;
		handler: () => void;
	}[];
	dismissible?: boolean;
};

export class FlowHintSystem {
	private hints: Hint[] = [];
	private dismissedHints = new Set<string>();

	getAvailableHints(
		flowData: {
			entrypoint: string;
			nodes: Record<string, any>;
			filters: Record<string, any>;
		}
	): Hint[] {
		const hints: Hint[] = [];

		if (!flowData.entrypoint) {
			hints.push({
				id: 'no-entrypoint',
				type: 'entrypoint-not-set',
				severity: 'warning',
				title: 'No Entrypoint Set',
				message: 'Please set an entrypoint node to start the flow.',
				actions: [
					{
						label: 'Select Entrypoint',
						handler: () => {
							console.log('Navigate to entrypoint selection');
						}
					}
				],
				dismissible: false
			});
		}

		const nodeCount = Object.keys(flowData.nodes).length;
		if (nodeCount === 0) {
			hints.push({
				id: 'empty-flow',
				type: 'empty-state',
				severity: 'info',
				title: 'Flow is Empty',
				message: 'Create your first node to get started with building your flow.',
				actions: [
					{
						label: 'Add Node',
						handler: () => {
							console.log('Add node action');
						}
					}
				],
				dismissible: true
			});
		}

		let unconfiguredCount = 0;
		Object.entries(flowData.nodes).forEach(([id, node]) => {
			if (!node.class) {
				unconfiguredCount++;
			}
		});

		if (unconfiguredCount > 0) {
			hints.push({
				id: 'unconfigured-nodes',
				type: 'unconfigured',
				severity: 'warning',
				title: `${unconfiguredCount} Node${unconfiguredCount > 1 ? 's' : ''} Not Configured`,
				message: 'Some nodes need configuration to work properly.',
				dismissible: true
			});
		}

		return hints.filter((hint) => !this.dismissedHints.has(hint.id));
	}

	dismissHint(hintId: string): void {
		this.dismissedHints.add(hintId);
	}

	restoreHint(hintId: string): void {
		this.dismissedHints.delete(hintId);
	}

	clearDismissed(): void {
		this.dismissedHints.clear();
	}
}

export function createHintSystem(): FlowHintSystem {
	return new FlowHintSystem();
}
