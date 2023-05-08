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
	<div class="flex justify-between">
		<div class="flex flex-row">
			<div class="">
				
				<svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="w-6 h-6 stroke-1">
					<path stroke-linecap="round" stroke-linejoin="round" d="M6.429 9.75L2.25 12l4.179 2.25m0-4.5l5.571 3 5.571-3m-11.142 0L2.25 7.5 12 2.25l9.75 5.25-4.179 2.25m0 0L21.75 12l-4.179 2.25m0 0l4.179 2.25L12 21.75 2.25 16.5l4.179-2.25m11.142 0l-5.571 3-5.571-3" />
				</svg>
				<!-- <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="w-6 h-6">
					<path stroke-linecap="round" stroke-linejoin="round" d="M16.5 8.25V6a2.25 2.25 0 00-2.25-2.25H6A2.25 2.25 0 003.75 6v8.25A2.25 2.25 0 006 16.5h2.25m8.25-8.25H18a2.25 2.25 0 012.25 2.25V18A2.25 2.25 0 0118 20.25h-7.5A2.25 2.25 0 018.25 18v-1.5m8.25-8.25h-6a2.25 2.25 0 00-2.25 2.25v6" />
				</svg> -->
				
				
				
			</div>

			<span class="dark:invert text-blacktext float-left pl-8 {settingsTrayOpen ? "text-columbia dark:text-inverted-columbia font-extrabold" : ""}">
				{entry_data.entry_name}
			</span>
		</div>

		<div>
			<SettingsTrayButton entryData={entry_data} bind:settingsTrayOpen/>

		</div>
	</div>
{/if}



<style>
	/* span { */
		/* background: url(/icons8-king-of-spades-50.png) 0 .25em no-repeat; */
		/* background-size: 16px; */
	/* } */
	
</style>