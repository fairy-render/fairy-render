import { defineConfig } from "@fairy-render/build";

export default defineConfig(() => {
	return {
		entry: {
			solid: {
				client: "src/solid/entry-client.tsx",
				server: "src/solid/entry-server.tsx",
			},
			react: {
				client: "src/react/entry-client.tsx",
				server: "src/react/entry-server.tsx",
			},
		},
		preset: {
			solid: {},
			react: {},
		},
	};
});
