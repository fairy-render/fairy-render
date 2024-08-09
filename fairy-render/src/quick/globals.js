((global) => {
	const files = [];
	const Fairy = {
		import: (fn, key) => {
			files.push(key);
			return fn();
		},
		runMain: async (path, ...args) => {
			files.length = 0;

			const { default: render } = await import(path);

			if (typeof render !== "function") {
				throw new TypeError("module does not export function");
			}

			const ret = await Promise.resolve(render(...args));

			if (typeof ret === "string") {
				return {
					content: ret,
					head: [],
					files: files.slice(),
				};
			}
			return {
				...ret,
				files: files.slice(),
			};
		},
	};

	Object.defineProperty(global, "Fairy", {
		value: Fairy,
	});
})(globalThis);
