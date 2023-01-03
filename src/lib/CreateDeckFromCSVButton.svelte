<script>
	import { open } from '@tauri-apps/api/dialog';
	// Open a selection dialog for image files
	let message = '';
	const handleOpen = async () => {
		const selected = await open({
			multiple: false,
			filters: [
				{
					name: 'Create Deck',
					extensions: ['txt', 'csv']
				}
			]
		});
		console.log(selected);
		if (Array.isArray(selected)) {
			message = 'Please only select one file';
			console.log(selected);
		} else if (selected === null) {
			// user cancelled the selection
			message = 'canceled';
			console.log(selected);
		} else {
			message = selected;
			console.log(selected);
		}
	};
</script>

<div>
	<button on:click={handleOpen}>Create Deck From CSV</button>
	<p>{message}</p>
</div>
