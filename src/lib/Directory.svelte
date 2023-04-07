<script lang="ts">
	import {slide} from 'svelte/transition'
	import SettingsTrayButton from './SettingsTray.svelte'
	import Deck from './Deck.svelte'
	import folderSystemStore from '$lib/stores/folderSystemStore'
	import type { EntryData } from '$lib/stores/folderSystemStore'
	import { invoke } from '@tauri-apps/api/tauri';

	let slideDuration = 125;
	let settingsTrayOpen: boolean = false;

	export let id: number;
	let entry_data: EntryData;
	let child_entries: EntryData[] = []

	function get_entry_data() {
		for (let entry of $folderSystemStore.data) {
			if (entry.entry_id == id) {
				entry_data = entry;
				break;
			}
		}

		get_child_entries();
	}
	get_entry_data();


	function get_child_entries() {
		for (let pair of $folderSystemStore.pairs) {
			if (pair.parent_id == id) {
				for (let entry of $folderSystemStore.data) {
					if (entry.entry_id == pair.child_id)
						child_entries.push(entry);
				}
			}
		}
	}

	
	async function toggleExpanded() {
		if (entry_data) {
			entry_data.is_expanded = !entry_data.is_expanded;
		}
		await invoke("toggle_is_expanded", { "entryId": entry_data.entry_id, "isExpanded": entry_data.is_expanded});
	}

</script>


	<!-- folder container -->
{#if entry_data}
	<div class="w-full"> 
		<div class="flow-root h-8">
			<div class="absolute">
				{#if entry_data.is_expanded}
					{#if entry_data.entry_type == "folder"}
						<!-- folder-open -->
						<svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="w-6 h-6">
							<path stroke-linecap="round" stroke-linejoin="round" d="M3.75 9.776c.112-.017.227-.026.344-.026h15.812c.117 0 .232.009.344.026m-16.5 0a2.25 2.25 0 00-1.883 2.542l.857 6a2.25 2.25 0 002.227 1.932H19.05a2.25 2.25 0 002.227-1.932l.857-6a2.25 2.25 0 00-1.883-2.542m-16.5 0V6A2.25 2.25 0 016 3.75h3.879a1.5 1.5 0 011.06.44l2.122 2.12a1.5 1.5 0 001.06.44H18A2.25 2.25 0 0120.25 9v.776" />
						</svg>
					{:else if entry_data.entry_type == "deadline"}
						<!-- clock-open -->
						<svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="w-6 h-6">
							<path stroke-linecap="round" stroke-linejoin="round" d="M12 6v6h4.5m4.5 0a9 9 0 11-18 0 9 9 0 0118 0z" />
						</svg>
					{:else}
						<!-- star -->
						<svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="w-6 h-6">
							<path stroke-linecap="round" stroke-linejoin="round" d="M11.48 3.499a.562.562 0 011.04 0l2.125 5.111a.563.563 0 00.475.345l5.518.442c.499.04.701.663.321.988l-4.204 3.602a.563.563 0 00-.182.557l1.285 5.385a.562.562 0 01-.84.61l-4.725-2.885a.563.563 0 00-.586 0L6.982 20.54a.562.562 0 01-.84-.61l1.285-5.386a.562.562 0 00-.182-.557l-4.204-3.602a.563.563 0 01.321-.988l5.518-.442a.563.563 0 00.475-.345L11.48 3.5z" />
						</svg>
					  
					  
					{/if}

				{:else}
					{#if entry_data.entry_type == "folder"}
						<!-- folder-closed -->
						<svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="w-6 h-6">
							<path stroke-linecap="round" stroke-linejoin="round" d="M2.25 12.75V12A2.25 2.25 0 014.5 9.75h15A2.25 2.25 0 0121.75 12v.75m-8.69-6.44l-2.12-2.12a1.5 1.5 0 00-1.061-.44H4.5A2.25 2.25 0 002.25 6v12a2.25 2.25 0 002.25 2.25h15A2.25 2.25 0 0021.75 18V9a2.25 2.25 0 00-2.25-2.25h-5.379a1.5 1.5 0 01-1.06-.44z" />
						</svg>
					{:else if entry_data.entry_type == "deadline"}
						<!-- clock-closed -->
						<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="currentColor" class="w-6 h-6">
							<path fill-rule="evenodd" d="M12 2.25c-5.385 0-9.75 4.365-9.75 9.75s4.365 9.75 9.75 9.75 9.75-4.365 9.75-9.75S17.385 2.25 12 2.25zM12.75 6a.75.75 0 00-1.5 0v6c0 .414.336.75.75.75h4.5a.75.75 0 000-1.5h-3.75V6z" clip-rule="evenodd" />
						</svg>

					{:else}
						<!-- star -->
						<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="currentColor" class="w-6 h-6">
							<path fill-rule="evenodd" d="M10.788 3.21c.448-1.077 1.976-1.077 2.424 0l2.082 5.007 5.404.433c1.164.093 1.636 1.545.749 2.305l-4.117 3.527 1.257 5.273c.271 1.136-.964 2.033-1.96 1.425L12 18.354 7.373 21.18c-.996.608-2.231-.29-1.96-1.425l1.257-5.273-4.117-3.527c-.887-.76-.415-2.212.749-2.305l5.404-.433 2.082-5.006z" clip-rule="evenodd" />
						</svg>
						  
					{/if}
				{/if}
			</div>
				
			
			<span 
				class="font-bold cursor-pointer dark:invert text-blacktext float-left {entry_data.is_expanded ? "h-8" : "h-7"} {settingsTrayOpen ? "text-columbia dark:text-inverted-columbia font-extrabold" : ""}" on:click={toggleExpanded} on:keydown={toggleExpanded}
				style="padding: 0 0 0 2em;">
				{entry_data.entry_name}
			</span>
			<SettingsTrayButton entryData={entry_data} bind:settingsTrayOpen />
		</div>

		{#if entry_data.is_expanded && child_entries.length > 0}
			<ul transition:slide={{duration:slideDuration}} class="">		
				{#each child_entries as entry}
					<li class="max-w-2xl lg:max-w-2xl {entry_data.entry_type == "deadline" ? "-mt-1" : ""}">
						{#if entry.entry_type == 'deck'}
							<Deck id={entry.entry_id} />
						{:else if entry_data.entry_type == "folder" && entry.entry_type == "folder" || entry.entry_type == "deadline" || entry.entry_type == "ankibox"}
							<svelte:self id={entry.entry_id} />
						{:else}
							{console.error("invalid entry")}
						{/if}
					</li>
				{/each}
			</ul>
		{/if}
	</div>
{/if}

<style lang="postcss">
	ul {
		padding: 0.3em 0 0 0.8em;
		margin: 0 0 0 1.4em;
	}

	li {
		padding: 0.2em 0;
	}
</style>
