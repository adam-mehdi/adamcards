
<script lang="ts">
	import {clickOutside} from '$lib/actions/click_outside.js';
    import { invoke } from '@tauri-apps/api/tauri';
	import fsStore from '$lib/stores/fsStore'

	type FileSystemObject = {
		entity_type: 'folder' | 'deadline' | 'deck';
		name: string;
		files: FileSystemObject[] | null;
		expanded: boolean | null;
		deadline_date: string | null;
		deadline_time: string | null;
	};

    export let entryType: string;
	export let path: string;
	$: newPath = path.replaceAll("~~", "/");


	// click on gear
	let settingsTrayOpen: boolean = false;
	function toggleSettingsTray() {
		settingsTrayOpen = !settingsTrayOpen;
	}

	function handleClickOutside() {
		settingsTrayOpen = false;
		createDeadlineTrayOpen = false;
		createFolderTrayOpen = false;
		createDeckTrayOpen = false;
		renameTrayOpen = false;
		moveTrayOpen = false;
	}

	// specialty trays
	let newName: string = "";

	let createFolderTrayOpen = false;
	let createDeckTrayOpen = false;
	let createDeadlineTrayOpen = false;

	let renameTrayOpen = false;
	let moveTrayOpen = false;

	$: actionTrayOpen = createFolderTrayOpen || createDeadlineTrayOpen || 
		createDeckTrayOpen || renameTrayOpen || moveTrayOpen;

	let deadlineDate: string | null = getNextWeekDate();
	let deadlineTime: string | null = "14:00:00";


	function getNextWeekDate(): string {
		let firstDay = new Date();
		let weekInMilliSeconds = 7 * 24 * 60 * 60 * 1000;
		let nextWeek = new Date(firstDay.getTime() + weekInMilliSeconds);

		const dd = String(nextWeek.getDate()).padStart(2, '0');
		const mm = String(nextWeek.getMonth() + 1).padStart(2, '0'); // January is 0!
		const yyyy = nextWeek.getFullYear();

		const date = yyyy + '-' + mm + '-' + dd ;

		return date;
	}

	// helper for functions that create entity
	function getDir(ancestors: string[]): FileSystemObject {
		let children: FileSystemObject[] = $fsStore;
		let entity: FileSystemObject = children[0];
		for (let name of ancestors) {
			const entries: FileSystemObject[] = children.filter((x: FileSystemObject) => x.name == name);

			if (entries.length == 0) {
				// create new folder at that path (automatically fires mkdir when mv)
				entries.push({
					entity_type: 'folder',
					name: name,
					files: [],
					expanded: true,
					deadline_date: null,
					deadline_time: null
				})
			} else if (entries.length > 1) {
				console.error("duplicate file system entity", name, path);
				break;
			} else {

				entity = entries.pop()!;
				children = entity.files!;
			}
		}
		return entity;
	}

	function rerender() {
		// re-render DOM
		const fs = $fsStore;
		$fsStore = fs;
	}

	// settings tray buttons
	async function handleCreateEntry() {
		let newEntityType: string = "";
		if (createFolderTrayOpen) 
			newEntityType = "folder";
		else if (createDeadlineTrayOpen) 
			newEntityType = "deadline";
		else 
			newEntityType = "deck";
		// get new entity by type open
		
		if (newName.length == 0)
			return;
			
		// get children of current folder
		const ancestors: string[] = path.split("~~");
		let children = getDir(ancestors).files!;

		if (children.filter((x: FileSystemObject) => x.name == newName).length > 0) {
			console.error("cannot have duplicate names");
			return;
		}

		// specify what type of entry is being created
		let newType: "folder" | "deadline" | "deck" = "folder";
		let newDate: string | null = null;
		let newTime: string | null = null;
		let newExpanded: boolean | null = null;
		if (newEntityType === "folder") {
			newType = "folder";
			newDate = null;
			newTime = null;
			newExpanded = true;
		} else if (newEntityType === "deadline") {
			invoke("create_deadline", { path, newName, deadlineDate, deadlineTime });
			newType = "deadline";
			newDate = deadlineDate;
			newTime = deadlineTime;
			newExpanded = true;
		} else {
			invoke("create_deck", { path, newName });
			newType = "deck";
			newDate = null;
			newTime = null;
			newExpanded = null;
		}


		let newEntry = {
				entity_type: newType,
				name: newName, // change this to result of input field
				files: [],
				expanded: newExpanded,
				deadline_date: newDate,
				deadline_time: newTime,
			}

		children.push(newEntry);

		handleCancel()
		rerender();
	}


	async function handleMove() {
		newPath = newPath.replaceAll("/", "~~");
		
		// get current entry
		let ancestors: string[] = path.split("~~");
		let oldName = ancestors.pop();

		// if path ends in `/`, keep current name
		if (newPath.endsWith("~~"))
			newPath = newPath + oldName;

		let oldParent = getDir(ancestors);
		let file: FileSystemObject = oldParent.files!
			.filter((x: FileSystemObject) => x.name == oldName)[0];
		

		// split out new path
		let newAncestors: string[] = newPath.split("~~");
		
		// return if invalid path
		if (newAncestors.length < 1) {
			handleCancel();
			rerender();
			return;
		}

		// remove entry from old parent path
		oldParent.files = oldParent.files!
			.filter((x: FileSystemObject) => x.name != oldName);		

		let newName = newAncestors.pop();
		file.name = newName!;
		let parent = getDir(newAncestors);
		parent.files!.push(file)

		let oldPath = path;
		path = newPath;
		await invoke("rename_entry", { path, oldPath })
		handleCancel();
		rerender();
		return;
		
	}

	async function handleRename() {
		// get current file
		let ancestors: string[] = path.split("~~");
		let file = getDir(ancestors);
		file.name = newName;

		ancestors.pop();
		ancestors.push(newName);
		const oldPath = path;
		path = ancestors.join("~~");

		await invoke("rename_entry", { path, oldPath })


		handleCancel();
		rerender();
	}

	function handleDelete() {
		settingsTrayOpen = false;

		// get parent of current file
		let ancestors: string[] = path.split("~~");
		let name = ancestors.pop();
		let parent = getDir(ancestors);

		// remove deleted file
		parent.files = parent.files!
			.filter((x: FileSystemObject) => x.name != name);		

		invoke("delete_entry", { path });

		handleCancel();
		rerender();
	}

	function handleCancel() {
		settingsTrayOpen = false;
		createFolderTrayOpen = false;
		createDeckTrayOpen = false;
		createDeadlineTrayOpen = false;

		renameTrayOpen = false;
		moveTrayOpen = false;

		newName = "";
		newPath = path.replaceAll("~~", "/");
		deadlineDate = getNextWeekDate();
		deadlineTime = "14:00:00";
	}

	function focus(el: any){
    	el.focus()
  	}

</script>


<div class="flex justify-end">
		<!-- Review -->
		<a href={`/${path}/review`}>
		<button class="float-right mr-5">
			<img class="w-4 h-4 hover:w-5 hover:h-5" src=eye.png alt="reviewing eye" />
		</button>
		</a>

		<!-- Edit -->
		<a href={`/${path}/edit`}>
		<button
			class="float-right mr-5">
			<img class="w-4 h-4 hover:w-5 hover:h-5" src=pencil.png alt="editing pencil" />
		</button>
		</a>

		{#if !settingsTrayOpen}
			<!-- Dropdown button -->
			<button
					class="float-right sm:mr-6"
					on:keydown|stopPropagation={toggleSettingsTray}
					on:click|stopPropagation={toggleSettingsTray}
				>
					<img class="h-4 w-4 hover:h-5 hover:w-5" src=settings-gear-black.png alt="setting gear" />
			</button>
		{:else}
			<!-- open carrot -->
			<button
					class="float-right">
					<img class="w-6 h-6" src=arrow-carrot-right.png alt="carrot pointing right" />
			</button>
		{/if}
</div>

{#if settingsTrayOpen}
<div
	class="settings-tray flex justify-between absolute {actionTrayOpen ? "w-64 mr-20" : "w-32 mr-52"} {entryType == "deck"? "" : "-mt-7"} rounded"
	use:clickOutside 
	on:click_outside={handleClickOutside}>

	{#if !actionTrayOpen}
	<!-- error on `click_outside` is due to svelte; like a necessary deprecation error, ignore it -->
	<ul class="p-0 m-0 text-sm">
		{#if entryType === "folder"}
		<li class="h-5 ">
			<button on:click={() => { createFolderTrayOpen = true; }}>
				Create Folder
			</button>
		</li>
		<hr />
		{/if}
		{#if entryType === "folder"}
		<li class="h-5 ">
			<button on:click={() => { createDeadlineTrayOpen = true; }}>
				Create Deadline
			</button>

		</li>
		<hr />
		{/if}
		{#if entryType === "deadline"}
		<li class="h-5 ">
			<button on:click={() => { createDeckTrayOpen = true; }}>
				Create Deck
			</button>
		</li>
		<hr />
		{/if}

		<li class="h-5 ">
			<button on:click={() => { renameTrayOpen = true; }}>
				Rename
			</button>
		</li>
		<hr />

		{#if entryType !== "deck" && path.includes("~~")}
		<li class="h-5 ">
			<button on:click={() => {moveTrayOpen = true; }}>
				Move
			</button>
		</li>
		{/if}
		<hr />

		<!-- cannot delete root directory -->
		{#if path.includes("~~")}
		<li class="h-5 ">
			<button class="" on:click={() => handleDelete() }>
				Delete
			</button>
		</li>
		{/if}
	</ul>

	<!-- open deadline tray -->
	{:else}

		<form class="w-full max-w-sm"
				on:submit|preventDefault={() => {
					settingsTrayOpen = false;
					if (renameTrayOpen) handleRename() 
					else if (moveTrayOpen) handleMove()
					else handleCreateEntry()
				}}>
			<div class="border-b border-teal-500 py-2 grid {createDeadlineTrayOpen ? "grid-rows-4" : "grid-rows-2" } grid-cols-3 gap-2">

				{#if !moveTrayOpen}
					<input type="text" use:focus placeholder="Enter Name" bind:value={newName} required class="h-5 col-span-3 bg-gray-200 appearance-none border-2 border-gray-200 rounded w-full py-2 px-4 text-gray-700 leading-tight focus:outline-none focus:bg-white focus:border-columbia placeholder:italic text-sm"/>
				{:else}
					<input bind:value={newPath} use:focus required class="h-5 col-span-3 bg-gray-200 appearance-none border-2 border-gray-200 rounded w-full py-2 px-4 text-gray-700 leading-tight focus:outline-none focus:bg-white focus:border-columbia"/>
				{/if}
				
				{#if createDeadlineTrayOpen}
					<input type="date" id="deadlineDate" bind:value={deadlineDate} required class="h-5 col-span-3 bg-gray-200 appearance-none border-2 border-gray-200 rounded w-full py-2 px-4 text-gray-700 leading-tight focus:outline-none focus:bg-white focus:border-columbia"/>
					<input type="time" bind:value={deadlineTime} required class="h-5 col-span-3 bg-gray-200 appearance-none border-2 border-gray-200 rounded w-full py-2 px-4 text-gray-700 leading-tight focus:outline-none focus:bg-white focus:border-columbia"/>
				{/if}

				<button type="submit" class="h-8 col-span-2 bg-white text-xs hover:bg-gray-100 text-gray-800 font-semibold py-2 px-4 border border-gray-400 rounded shadow">
					{renameTrayOpen ? "Rename" : "Create"}
				</button>
				<button type="button" on:click={ handleCancel	 } class="h-8 col-span-1 bg-white text-xs hover:bg-gray-100 text-gray-800 font-semibold py-2 px-4 border border-gray-400 rounded shadow">
					Cancel
				</button>
			</div>
		</form>

	{/if}
</div>
{/if}


<style>
	.settings-tray {
		padding: 5px;
		display: flex;
		align-items: flex-start;
		justify-content: center;
		right: 2em;
		top: 2em;
		box-shadow: 0px 0px 10px #eee;
		z-index: 4;
	}

</style>