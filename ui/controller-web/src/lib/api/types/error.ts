export type ErrorStackFrame = {
	typeName: string;
	error: string;
};

export type ErrorStack = {
	frames: ErrorStackFrame[];
};

export type ResultObject<T> =
	| {
			data: T;
	  }
	| {
			error: ErrorStack;
	  };
