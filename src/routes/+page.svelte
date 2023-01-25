<script lang="ts">

	import Folder from '$lib/Folder.svelte';
	import { invoke } from '@tauri-apps/api/tauri';
	import { onMount, onDestroy } from 'svelte';
	import fsStore from '$lib/stores/fsStore'

	type FileSystemObject = {
		entity_type: 'folder' | 'deadline' | 'deck';
		name: string;
		files: FileSystemObject[] | null;
		expanded: boolean | null;
		deadline_date: string | null;
		deadline_time: string | null;
	};

	let mode = "";

	let fs: FileSystemObject[] = [];
	async function load_file_system() {
		const root: FileSystemObject = await invoke('read_fs_json');
		// console.log(root);
		fs = [root]
		fsStore.update(() => { return fs; });
	}
	load_file_system();

	// re-render fs on DOM whenever fsStore changes
	$: fs = $fsStore;

	// write new folder structure `fs` to file system
	onDestroy(async () => {
		await invoke('write_fs_json', { fs });
	});

	function toggleDarkMode() {
		if (mode == "dark")
			mode = "light";
		else
			mode = "dark";
	}
	

</script>


<div class={mode}>
	<!-- bar at top -->
	<div class="dark:bg-gradient-to-r from-[#B9D9EB] to-[#00693e] bg-[#e1dfdd] h-screen">

		
		<div class="flex justify-between mb-10 mx-10">
			<h1 class="flex-none font-serif text-xl font-bold mt-5">

			</h1>


			<!-- dark mode button -->
			<div class="fled justify-evenly mt-5">
			{#if mode == "dark"}
				<svg 
					on:click={toggleDarkMode} 
					class="flex-none h-7 w-7 cursor-pointer" 
					viewBox="0 0 20 20" xmlns="http://www.w3.org/2000/svg" aria-hidden="true" data-darkreader-inline-fill="">
					<path clip-rule="evenodd" fill-rule="evenodd" d="M7.455 2.004a.75.75 0 01.26.77 7 7 0 009.958 7.967.75.75 0 011.067.853A8.5 8.5 0 116.647 1.921a.75.75 0 01.808.083z"></path>
				</svg>
			{:else}
				<svg 
					on:click={toggleDarkMode} 
					class="flex-none h-7 w-7 cursor-pointer" 
					viewBox="0 0 20 20" xmlns="http://www.w3.org/2000/svg" aria-hidden="true" data-darkreader-inline-fill="">
					<path d="M10 2a.75.75 0 01.75.75v1.5a.75.75 0 01-1.5 0v-1.5A.75.75 0 0110 2zM10 15a.75.75 0 01.75.75v1.5a.75.75 0 01-1.5 0v-1.5A.75.75 0 0110 15zM10 7a3 3 0 100 6 3 3 0 000-6zM15.657 5.404a.75.75 0 10-1.06-1.06l-1.061 1.06a.75.75 0 001.06 1.06l1.06-1.06zM6.464 14.596a.75.75 0 10-1.06-1.06l-1.06 1.06a.75.75 0 001.06 1.06l1.06-1.06zM18 10a.75.75 0 01-.75.75h-1.5a.75.75 0 010-1.5h1.5A.75.75 0 0118 10zM5 10a.75.75 0 01-.75.75h-1.5a.75.75 0 010-1.5h1.5A.75.75 0 015 10zM14.596 15.657a.75.75 0 001.06-1.06l-1.06-1.061a.75.75 0 10-1.06 1.06l1.06 1.06zM5.404 6.464a.75.75 0 001.06-1.06l-1.06-1.06a.75.75 0 10-1.061 1.06l1.06 1.06z"></path>
				</svg>
			{/if}
			</div>
		</div>
			
	<!-- folder system -->
		<div class="mx-auto max-w-xl pr-16 pl-2 lg:ml-56 border-y border-gray-300 rounded-lg">
			{#each fs as entity (entity)}
				<Folder 
					name={entity.name} 
					bind:files={entity.files} 
					expanded={entity.expanded} 
					path={entity.name}
				/>
			{/each}
		</div>
	</div>
</div>
