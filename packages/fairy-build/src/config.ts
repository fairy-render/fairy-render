import type { PluginOption } from "vite";
import Presets, {
	type Cmd,
	type Preset,
	type PresetOptionsType,
} from "./presets.js";
import type { Options } from "./shared.js";
import Path from "node:path";

export type DefineFn = () => UserConfig;

export const DefaultPort = "3768";

export interface Entry {
	client: string;
	server: string;
}

export function isEntry(a: unknown): a is Entry {
	return (
		!!a &&
		typeof (a as unknown as Record<string, string>).client === "string" &&
		typeof (a as unknown as Record<string, string>).server === "string"
	);
}

export type EntryPoint = string | Entry | Record<string, Entry | string>;

export type PresetOptions =
	| Preset
	| { [K in Preset]: PresetOptionsType<(typeof Presets)[K]> };

export async function resolvePresets(
	cmd: Cmd,
	preset?: PresetOptions,
): Promise<PluginOption[]> {
	if (!preset) return [];

	const output: PluginOption[] = [];

	const push = async <T extends Preset>(
		key: T,
		options?: PresetOptionsType<T>,
	) => {
		const p = Presets[key as Preset];
		if (!p) throw new Error(`preset "${key}" not found`);

		output.push(...(await Promise.resolve(p(cmd, options ?? {}))));
	};

	if (typeof preset === "string") {
		await push(preset);
	} else {
		for (const k in preset) {
			await push(k as Preset);
		}
	}

	return output;
}

export interface UserConfig {
	entry: EntryPoint;
	assets?: string;
	outputDir?: string;
	preset?: PresetOptions;
	plugins?: PluginOption[];
	base?: string;
}

export class FairyConfig {
	#inner: DefineFn;
	constructor(cfg: DefineFn) {
		this.#inner = cfg;
	}

	resolve() {
		return Promise.resolve(this.#inner());
	}
}

export interface RuntimeOptions {
	assets: string;
	outputDir: string;
	entry: EntryPoint;
	preset?: PresetOptions | undefined;
	plugins?: PluginOption[] | undefined;
	port: number;
	root: string;
	base: string;
}

export function resolveRuntimeConfig(
	cfg: UserConfig,
	options: Options,
): RuntimeOptions {
	const port = Number.parseInt(options.port ?? DefaultPort);

	return {
		...cfg,
		assets: cfg.assets ?? "assets",
		outputDir: cfg.outputDir ?? "dist",
		port,
		root: Path.resolve(options.workDir ?? process.cwd()),
		base: cfg.base ?? "/",
	};
}
