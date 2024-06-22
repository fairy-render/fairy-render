import { defineConfig } from "@fairy-render/build";

export default defineConfig(() => {
	return {
		entry: {
			client: "src/entry-client.tsx",
			server: "src/entry-server.tsx",
		},
		preset: {
			react: {},
		},
	};
});
