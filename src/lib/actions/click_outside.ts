export function clickOutside(node: HTMLSpanElement, foldersMuted: boolean) {
	// eslint-disable-next-line @typescript-eslint/no-explicit-any
	const handleClick = (event: { target: any; stopPropagation: () => void }) => {
		if (!node.contains(event.target)) {
			if (foldersMuted) {
				event.stopPropagation();
			}
			node.dispatchEvent(new CustomEvent('outclick'));
		}
	};

	document.addEventListener('click', handleClick, true);

	return {
		destroy() {
			document.removeEventListener('click', handleClick, true);
		}
	};
}
