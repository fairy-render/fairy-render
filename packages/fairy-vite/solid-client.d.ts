declare type Element =
	| Node
	| ArrayElement
	| (string & {})
	| number
	| boolean
	| null
	| undefined;
declare interface ArrayElement extends Array<Element> {}

export declare function render(render: () => Element, el: HTMLElement): void;
