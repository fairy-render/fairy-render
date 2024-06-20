import type { Options } from "vite-plugin-solid";

export type Cmd = "build" | "watch";

export type PresetFn<T> = (cmd: Cmd, options: T) => unknown;

export type PresetOptionsType<T> = T extends PresetFn<infer K> ? K : never;

const Presets = {
	solid: async (command: "build" | "watch", options: Partial<Options>) => {
		const solid = await import("vite-plugin-solid");
		return solid.default({
			...options,
			ssr: command === "build",
		});
	},
};

export type Preset = keyof typeof Presets;

export default Presets;
