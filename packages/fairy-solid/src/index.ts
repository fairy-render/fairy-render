import type { JSX } from "solid-js";
import {
	render as solidRender,
	hydrate,
	isDev,
	type MountableElement,
} from "solid-js/web";

export function render(
	app: () => JSX.Element,
	element: MountableElement,
): () => void {
	if (isDev) {
		return solidRender(app, element);
	}
	return hydrate(app, element);
}
