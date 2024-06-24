import type { ReactNode } from "react";
import { createRoot, type Root } from "react-dom/client";

export function render(node: ReactNode, element: Element): Root {
	const root = createRoot(element);
	root.render(node);
	return root;
}
