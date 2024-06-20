import { defineConfig } from "tsup";

export default defineConfig({
	entry: ["./src/index.ts", "./src/solid.ts"],
	format: ["cjs", "esm"],
	dts: true,
	clean: true,
});
