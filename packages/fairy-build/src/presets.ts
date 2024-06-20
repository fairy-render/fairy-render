import type { Options as SolidOptions } from "vite-plugin-solid";
import type { Options as ReactOptions } from "@vitejs/plugin-react";

export type Cmd = "build" | "watch";

export type PresetFn<T> = (cmd: Cmd, options: T) => unknown;

export type PresetOptionsType<T> = T extends PresetFn<infer K> ? K : never;

const Presets = {
	solid: async (command: "build" | "watch", options: Partial<SolidOptions>) => {
		const solid = await import("vite-plugin-solid");
		return solid.default({
			...options,
			ssr: command === "build",
		});
	},
	react: async (_command: Cmd, options: Partial<ReactOptions>) => {
		const react = await import("@vitejs/plugin-react");
		return react.default(options);
	},
};

export type Preset = keyof typeof Presets;

export default Presets;
