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

	const ofetch = global.fetch;

	Object.defineProperty(global, "Fairy", {
		value: Fairy,
		fetch: (url, opts) => {
			if (typeof url === "string" && url.startsWith("/")) {
				url = `internal://internal${url}`;
			}

			return ofetch(url, opts);
		},
	});
})(globalThis);
