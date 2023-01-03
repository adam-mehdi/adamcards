<script lang="ts">
	interface DeckEntry {
		name: String;
		deadlineString: String;
		deadlineDt: Date;
	}

	let deadline = new Date('2023-02-20 22:22');
	console.log(deadline.toISOString().slice(0, -5));

	let testData: DeckEntry = {
		name: 'Test Deck',
		deadlineString: '2023-02-20T22:22',
		deadlineDt: deadline
	};

	import { invoke } from '@tauri-apps/api/tauri';

	const handleClick = () => {
		invoke('pde', {
			deckEntry: {
				name: 'test deck',
				ndt: deadline.toISOString().slice(0, -5)
			}
		});
	};
</script>

<button on:click={handleClick}>Test</button>
