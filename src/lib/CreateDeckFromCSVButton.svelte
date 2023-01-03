<script>
	import { invoke } from '@tauri-apps/api/tauri';
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
			console.log(selected);
			// Send Path To Backend
			try {
				message = await invoke('create_card_from_csv', { cardPath: selected });
			} catch (err) {
				// The promise rejection returns a string, but there's no way to tell typescript this
				// so the following logic is required to assign the err to message
				if ( typeof err === 'string') {
					message = err; 
				} else {
					// This should never run, but it's here just in case...
					message = "error opening file and error returned is of unexpected type, see console for details.";
					console.error(err);
				}
			}
		}

	};
</script>

<div>
	<button on:click={handleOpen}>Create Deck From CSV</button>
	<p>{message}</p>
</div>
