<script lang="ts">
	// import SettingsTrayButton from './SettingsTray.svelte';
	import folderSystemStore from '$lib/stores/folderSystemStore'
	import type { EntryData } from '$lib/stores/folderSystemStore'
	import SettingsTrayButton from './SettingsTray.svelte'

	export let id: number;
	let entry_data: EntryData;
	let settingsTrayOpen = false;

	function get_entry_data() {
		for (let entry of $folderSystemStore.data) {
			if (entry.entry_id == id) {
				entry_data = entry;
				break;
			}
		}
	}
	get_entry_data();


</script>

{#if entry_data}
	<div class="flow-root">
		<div class="absolute">
			<svg class="w-6 h-6" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor">
				<path stroke-linecap="round" stroke-linejoin="round" d="M19.5 14.25v-2.625a3.375 3.375 0 00-3.375-3.375h-1.5A1.125 1.125 0 0113.5 7.125v-1.5a3.375 3.375 0 00-3.375-3.375H8.25m2.25 0H5.625c-.621 0-1.125.504-1.125 1.125v17.25c0 .621.504 1.125 1.125 1.125h12.75c.621 0 1.125-.504 1.125-1.125V11.25a9 9 0 00-9-9z" />
			</svg>
		</div>

		<span class="dark:invert text-blacktext float-left pl-8 {settingsTrayOpen ? "text-columbia dark:text-inverted-columbia font-extrabold" : ""}">
			{entry_data.entry_name}
		</span>

		<SettingsTrayButton entryData={entry_data} bind:settingsTrayOpen/>
	</div>
{/if}



<style>
	/* span { */
		/* background: url(/icons8-king-of-spades-50.png) 0 .25em no-repeat; */
		/* background-size: 16px; */
	/* } */
	
</style>