import { defineConfig } from "tsup";

export default defineConfig({
	entry: ["./src/index.ts", "./src/index.dev.ts", "./src/server.ts"],
	format: ["cjs", "esm"],
	dts: true,
	clean: true,
});
