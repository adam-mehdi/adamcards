<script>
	import { invoke } from '@tauri-apps/api/tauri';

	const timezones = [
		{ letter: 'A', offsetString: 'UTC+01:00', stampOffset: '+01:00' },
		{ letter: 'B', offsetString: 'UTC+02:00', stampOffset: '+02:00' },
		{ letter: 'C', offsetString: 'UTC+03:00', stampOffset: '+03:00' },
		{ letter: 'D', offsetString: 'UTC+04:00', stampOffset: '+04:00' },
		{ letter: 'E', offsetString: 'UTC+05:00', stampOffset: '+05:00' },
		{ letter: 'F', offsetString: 'UTC+06:00', stampOffset: '+06:00' },
		{ letter: 'G', offsetString: 'UTC+07:00', stampOffset: '+07:00' },
		{ letter: 'H', offsetString: 'UTC+08:00', stampOffset: '+08:00' },
		{ letter: 'I', offsetString: 'UTC+09:00', stampOffset: '+09:00' },
		{ letter: 'K', offsetString: 'UTC+10:00', stampOffset: '+10:00' },
		{ letter: 'L', offsetString: 'UTC+11:00', stampOffset: '+11:00' },
		{ letter: 'M', offsetString: 'UTC+12:00', stampOffset: '+12:00' },
		{ letter: 'N', offsetString: 'UTC-01:00', stampOffset: '-01:00' },
		{ letter: 'O', offsetString: 'UTC-02:00', stampOffset: '-02:00' },
		{ letter: 'P', offsetString: 'UTC-03:00', stampOffset: '-03:00' },
		{ letter: 'Q', offsetString: 'UTC-04:00', stampOffset: '-04:00' },
		{ letter: 'R', offsetString: 'UTC-05:00', stampOffset: '-05:00' },
		{ letter: 'S', offsetString: 'UTC-06:00', stampOffset: '-06:00' },
		{ letter: 'T', offsetString: 'UTC-07:00', stampOffset: '-07:00' },
		{ letter: 'U', offsetString: 'UTC-08:00', stampOffset: '-08:00' },
		{ letter: 'V', offsetString: 'UTC-09:00', stampOffset: '-09:00' },
		{ letter: 'W', offsetString: 'UTC-10:00', stampOffset: '-10:00' },
		{ letter: 'X', offsetString: 'UTC-11:00', stampOffset: '-11:00' },
		{ letter: 'Y', offsetString: 'UTC-12:00', stampOffset: '-12:00' },
		{ letter: 'Z', offsetString: 'UTC+00:00', stampOffset: 'Z' }
	];

	let inputText = '';
	let deadlineDate = '';
	let deadlineTime = '';
	let deckName = '';
	let deadlineOffset = '';

	const handleSubmit = () => {
		const newDeck = {
			name: deckName,
			deadline_string: `${deadlineDate}T${deadlineTime}${deadlineOffset}`, // can't be camel cased...
			text: inputText
		};
		console.log(newDeck);
		invoke('create_deck', { deckInfo: newDeck });
	};
</script>

<a href="/"><button>back</button></a>
<br />
<br />
<div>
	<form on:submit|preventDefault={handleSubmit}>
		<label>
			Deadline:
			<br />
			<input type="date" bind:value={deadlineDate} style:width={'150px'} required />
			<input type="time" bind:value={deadlineTime} style:width={'150px'} required />
			<select bind:value={deadlineOffset} style:width={'150px'} required>
				{#each timezones as timezone}
					<option value={timezone.stampOffset}>
						{`${timezone.letter}: ${timezone.offsetString}`}
					</option>
				{/each}
			</select>
		</label>
		<br />
		<br />
		<label>
			Deck Name:
			<br />
			<input type="text" bind:value={deckName} required />
		</label>
		<br />
		<br />
		<label>
			Text:
			<br />
			<textarea bind:value={inputText} cols={80} rows={20} required />
		</label>
		<br />
		<br />
		<button type="submit">Create Deck</button>
	</form>
</div>
