type DebugFn = ((...args: unknown[]) => void) & {
	enabled: boolean;
	namespace?: string;
	extend: (namespace: string) => DebugFn;
	destroy: () => void;
};

function createDebug(namespace?: string): DebugFn {
	const debug = ((..._args: unknown[]) => {}) as DebugFn;
	debug.enabled = false;
	debug.namespace = namespace;
	debug.extend = (suffix: string) =>
		createDebug(namespace ? `${namespace}:${suffix}` : suffix);
	debug.destroy = () => {};
	return debug;
}

export default createDebug;
