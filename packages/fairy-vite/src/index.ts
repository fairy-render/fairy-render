import type { Plugin, ConfigEnv } from "vite";
import Path from "node:path";

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
}: Options = {}): Plugin {
	const filterRe = new RegExp(`\\.(?:${extensions.join("|")})$`);

	let environ: ConfigEnv;

	const resolveDir = (env: ConfigEnv, ssr: boolean, path?: string) => {
		const sec = ssr ? "server" : "client";
		return path ? Path.join(path, sec) : Path.join("dist", sec);
	};

	return {
		name: "fairy",
		config(config, env) {
			const ssr = env.ssrBuild || (env as any).isSsrBuild;
			return {
				ssr: {
					noExternal: true,
				},
				build: {
					manifest: true,
					ssrManifest: !ssr,
					outDir: resolveDir(env, ssr, config.build?.outDir),
				},
			};
		},

		renderDynamicImport(options) {
			if (
				!options.targetModuleId ||
				!(environ.ssrBuild || (environ as any).isSsrBuild)
			)
				return;
			const key = options.targetModuleId
				?.replace(process.cwd(), "")
				.substring(1);

			return {
				left: "Fairy.import(() => import(",
				right: `), "${key}")`,
			};
		},
		apply(config, env) {
			environ = env;
			return env.command === "build";
		},
	};
}
