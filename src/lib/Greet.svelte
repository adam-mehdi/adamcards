<script lang="ts">
	import { invoke } from '@tauri-apps/api/tauri';

	let name = '';
	let greetMsg = '';
	let specialInfo = '';

	async function greet() {
		greetMsg = await invoke('greet', { name });
	}

	async function updateSpecialInfo() {
		await invoke('update_special_info', { specialInfo });
	}
</script>

<div>
	<input id="special-info-input" placeholder="Enter a secret..." bind:value={specialInfo} />
	<button on:click={updateSpecialInfo}>Greet</button>
</div>

<div>
	<input id="greet-input" placeholder="Enter a name..." bind:value={name} />
	<button on:click={greet}>Greet</button>
	<p>{greetMsg}</p>
</div>
