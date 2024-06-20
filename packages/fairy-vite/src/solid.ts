import { createFilter, type UserConfig, type Plugin } from "vite";

const defaultExtensions = [
	"js",
	"cjs",
	"ts",
	"tsx",
	"jsx",
	"mjs",
	"mts",
	"mtsx",
];

export interface Options {
	extensions?: string[];
	include?: string[];
	exclude?: string[];
}

export default function ({
	extensions = defaultExtensions,
	include = [],
	exclude = [],
}: Options = {}): Plugin {
	const filterRe = new RegExp(`\\.(?:${extensions.join("|")})$`);

	const filter = createFilter([filterRe, include].flat(), exclude);

	let config: Partial<UserConfig> & { command: string };

	const virtualModuleId = "@fairy/solid-client";
	const resolvedVirtualModuleId = `\0${virtualModuleId}`;

	return {
		name: "fairy-solid",
		configResolved(cfg) {
			config = cfg as any;
		},
		resolveId(source, importer, options) {
			if (source === virtualModuleId) {
				return resolvedVirtualModuleId;
			}
		},
		load(id, options) {
			if (id !== resolvedVirtualModuleId) {
				return;
			}

			return config.command === "build"
				? `export { hydrate as render } from 'solid-js/web'`
				: `export { render } from 'solid-js/web'`;
		},
	};
}
