type ExtendTarget = Record<PropertyKey, unknown>;

function isObjectLike(value: unknown): value is ExtendTarget {
	return typeof value === "object" && value !== null;
}

function isPlainObject(value: unknown): value is ExtendTarget {
	if (!isObjectLike(value)) return false;
	const prototype = Object.getPrototypeOf(value);
	return prototype === Object.prototype || prototype === null;
}

function cloneValue(value: unknown, deep: boolean): unknown {
	if (!deep) return value;
	if (Array.isArray(value)) {
		return value.map((item) => cloneValue(item, true));
	}
	if (isPlainObject(value)) {
		return extend(true, {}, value);
	}
	return value;
}

function assignValue(
	target: ExtendTarget,
	key: PropertyKey,
	value: unknown,
	deep: boolean,
) {
	if (deep && Array.isArray(value)) {
		target[key] = cloneValue(value, true);
		return;
	}

	if (deep && isPlainObject(value)) {
		const existing = target[key];
		const base = isPlainObject(existing) ? existing : {};
		target[key] = extend(true, base, value);
		return;
	}

	if (value !== undefined) {
		target[key] = value;
	}
}

function extend(...args: unknown[]): ExtendTarget {
	let deep = false;
	let index = 0;

	if (typeof args[0] === "boolean") {
		deep = args[0];
		index = 1;
	}

	const initialTarget = args[index];
	const target: ExtendTarget =
		isObjectLike(initialTarget) || typeof initialTarget === "function"
			? (initialTarget as ExtendTarget)
			: {};

	for (let sourceIndex = index + 1; sourceIndex < args.length; sourceIndex += 1) {
		const source = args[sourceIndex];
		if (!isObjectLike(source)) continue;

		for (const key of Reflect.ownKeys(source)) {
			const value = (source as ExtendTarget)[key];
			assignValue(target, key, value, deep);
		}
	}

	return target;
}

export default extend;
