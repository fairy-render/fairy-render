import type { ReactNode } from "react";
import {
	type RenderToReadableStreamOptions,
	renderToReadableStream,
} from "react-dom/server.browser";

const decoder = new TextDecoder();

export default async function render(
	node: ReactNode,
	opts: RenderToReadableStreamOptions,
) {
	const html = await renderToReadableStream(node, opts);

	const reader = html.getReader();

	const output = [];

	while (true) {
		const { done, value } = await reader.read();
		if (value) {
			output.push(...value);
		}
		if (done) {
			break;
		}
	}

	const buffer = new Uint8Array(output);

	return { content: decoder.decode(buffer.buffer), head: [] };
}
