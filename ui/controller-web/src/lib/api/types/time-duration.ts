export type TimeDuration = number | string | 'Never';

export type TimeDurationKind =
	| {
			kind: 'Millisecond';
			value: number;
	  }
	| {
			kind: 'Expr';
			value: {
				hour?: number;
				minute?: number;
				second?: number;
				millisecond?: number;
			};
	  }
	| {
			kind: 'Never';
	  };

export function parseTimeDurationFromString(input: string): TimeDurationKind | Error {
	if (input === 'Never') {
		return { kind: 'Never' };
	} else if (input.match(/^\d+$/)) {
		return { kind: 'Millisecond', value: parseInt(input, 10) };
	} else {
		const match = input.match(/(?:(\d+)h)?(?:(\d+)m)?(?:(\d+)s)?(?:(\d+)ms)?/);
		if (match && match[0] !== '') {
			return {
				kind: 'Expr',
				value: {
					hour: match[1] ? parseInt(match[1], 10) : undefined,
					minute: match[2] ? parseInt(match[2], 10) : undefined,
					second: match[3] ? parseInt(match[3], 10) : undefined,
					millisecond: match[4] ? parseInt(match[4], 10) : undefined,
				}
			};
		} else {
			return new Error(`Invalid time duration format: ${input}`);
		}
	}
}

export function formatTimeDuration(duration: TimeDurationKind): TimeDuration {
	switch (duration.kind) {
		case 'Millisecond':
			return duration.value;
		case 'Expr': {
			const parts: string[] = [];
			if (duration.value.hour) parts.push(`${duration.value.hour}h`);
			if (duration.value.minute) parts.push(`${duration.value.minute}m`);
			if (duration.value.second) parts.push(`${duration.value.second}s`);
			if (duration.value.millisecond) parts.push(`${duration.value.millisecond}ms`);
			return parts.join('') ?? undefined;
		}
		case 'Never':
			return 'Never';
	}
}
