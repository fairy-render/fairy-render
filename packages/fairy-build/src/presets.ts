import type { Options as SolidOptions } from "vite-plugin-solid";
import type { Options as ReactOptions } from "@vitejs/plugin-react";
import type { PluginOption } from "vite";

export type Cmd = "build" | "watch";

export type PresetFn<T> = (cmd: Cmd, options: T) => unknown;

export type PresetOptionsType<T> = T extends PresetFn<infer K> ? K : never;

const Presets = {
	solid: async (command: "build" | "watch", options: Partial<SolidOptions>) => {
		const solid = await import("vite-plugin-solid");
		return [
			solid.default({
				...options,
				ssr: command === "build",
			}),
		] as PluginOption[];
	},
	react: async (command: Cmd, options: Partial<ReactOptions>) => {
		const react = await import("@vitejs/plugin-react");
		return [
			{
				name: "@fairy/build/react",
				config(userConfig) {
					const replaceDev = command === "watch";
					return {
						resolve: {
							conditions: [
								"@fairy-render/react",
								...(replaceDev ? ["development"] : []),
								...(userConfig.mode === "test" && replaceDev
									? ["browser"]
									: []),
							],
						},
					};
				},
			},
			react.default({ ...options }),
		] as PluginOption[];
	},
};

export type Preset = keyof typeof Presets;

export default Presets;
