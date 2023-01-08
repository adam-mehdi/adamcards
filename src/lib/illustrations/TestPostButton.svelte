<script lang="ts">
	interface DeckEntry {
			name: String;
			deadlineString: String;
			deadlineDt: Date;
	}

	let deadlineString = '2023-02-20T22:22';
	let deadline = new Date(deadlineString);
	// console.log(deadline.toISOString().slice(0, -5));

	let testData: DeckEntry = {
		name: 'Test Deck',
		deadlineString: deadlineString,
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
