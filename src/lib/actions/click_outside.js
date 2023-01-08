export function clickOutside(node, foldersMuted) {
	const handleClick = (event) => {
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
