
<script lang="ts">
	import {clickOutside} from '$lib/actions/click_outside.js';
    import { invoke } from '@tauri-apps/api/tauri';
	import fsStore from '$lib/stores/fsStore'
	import quotasStore from './stores/quotasStore';
	import configStore from './stores/configStore';
	import deletingStore from './stores/deletingStore';
	import Hint from 'svelte-hint';



	type EntryQuota = {
		new_left: number,
		review_left: number,
		num_progressed: number,
		days_to_go: number,
		tot_days: number,
		deck_path: string
	};

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


	let quota: EntryQuota | null = null;
	function getQuota() {
		if ($quotasStore.length == 0) {
			return;
		}
			
		if (entryType == "deck")
			quota = $quotasStore.filter((x: EntryQuota) => x.deck_path == path)[0];

		else {
			let decks = $quotasStore
				.filter((x: EntryQuota) => x.deck_path.startsWith(path));
			quota = {
				new_left: decks.reduce((sum, current) => sum + current.new_left, 0),
				review_left: decks.reduce((sum, current) => sum + current.review_left, 0),
				num_progressed: decks.reduce((sum, current) => sum + current.num_progressed, 0),
				days_to_go: -1,
				tot_days: -1,
				deck_path: path
			}

		}
	}
	getQuota();



	// click on gear
	export let settingsTrayOpen = false;
	function toggleSettingsTray() {
		settingsTrayOpen = !settingsTrayOpen;
	}

	function handleClickOutside() {
		settingsTrayOpen = false;
		createDeadlineTrayOpen = false;
		resetDeadlineTrayOpen = false;
		createFolderTrayOpen = false;
		createDeckTrayOpen = false;
		renameTrayOpen = false;
		moveTrayOpen = false;
	}

	// specialty trays
	let newName: string = "";
	let studyIntensity = "";

	let createFolderTrayOpen = false;
	let createDeckTrayOpen = false;
	let createDeadlineTrayOpen = false;
	let resetDeadlineTrayOpen = false;

	let renameTrayOpen = false;
	let moveTrayOpen = false;

	$: actionTrayOpen = createFolderTrayOpen || createDeadlineTrayOpen || 
		createDeckTrayOpen || renameTrayOpen || moveTrayOpen || resetDeadlineTrayOpen;

	let deadlineDate: string | null = getNextWeekDate();
	let deadlineTime: string | null = "14:00:00";
	let entered_dup_name = false;


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

	// perform dfs over file system to get a list of the paths of decks 
	function dfs(curr_entry: FileSystemObject, curr_path: string) {
		if (curr_entry.entity_type == "deck")
			all_decks.push(curr_path)

		if (curr_entry.files == null)
			return;

		let i = 0;		
		while (curr_entry.files.length > i) {
			dfs(curr_entry.files[i], curr_path + "~~" + curr_entry.files[i].name);
			i += 1;
		}
	}

	let all_decks: string[] = [];
	function compute_has_deck() {
		let ancestors = path.split("~~");

		let curr_entry;
		if (ancestors.length == 1) {
			curr_entry = $fsStore[0];
		} else {
			let curr_dir = $fsStore;
			for (let entry of ancestors){
				let new_entry = curr_dir.filter((x) => x.name == entry);
				if (new_entry.length != 1) {
					console.error(curr_dir);
					console.error(entry);
					console.error(ancestors);
				}
				curr_dir = new_entry[0].files!;
				curr_entry = new_entry[0];
			}
			if (!curr_entry) {
				console.error("UNDEFINED");
				return;
			}

		}
		dfs(curr_entry, path)

		return all_decks.length > 0;
	}
	let has_deck = compute_has_deck();

	function rerender() {
		// re-render DOM
		const fs = $fsStore;
		$fsStore = fs;
	}

	// settings tray buttons
	async function handleCreateEntry() {
		
		if (newName.length == 0)
			return;
			
		entered_dup_name = get_is_dup_name(path, newName);
		if (entered_dup_name) {
			rerender();
			return;
		}

		// specify what type of entry is being created
		let newType: "folder" | "deadline" | "deck" = "folder";
		let newDate: string | null = null;
		let newTime: string | null = null;
		let newExpanded: boolean | null = null;
		if (createFolderTrayOpen) {
			newType = "folder";
			newDate = null;
			newTime = null;
			newExpanded = true;
		} else if (createDeadlineTrayOpen) {
			invoke("create_deadline", { path, newName, studyIntensity, deadlineDate, deadlineTime });
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

		const ancestors: string[] = path.split("~~");
		let children = getDir(ancestors).files!;
		children.push(newEntry);

		// saveDec
		rerender();
		await handleCancel()
		has_deck = compute_has_deck()
	}


	async function handleMove() {
		newPath = newPath.replaceAll("/", "~~");
		
		// split out new path
		let newAncestors: string[] = newPath.split("~~");

		// return if invalid path
		if (newAncestors.length < 1) 
			return;
		let newName = newAncestors.pop()!;

		entered_dup_name = get_is_dup_name(newAncestors.join("~~"), newName);
		if (entered_dup_name)
			return;
		
		// get current entry
		let ancestors: string[] = path.split("~~");
		let oldName = ancestors.pop();

		// if path ends in `/`, keep current name
		if (newPath.endsWith("~~"))
			newPath = newPath + oldName;

		let oldParent = getDir(ancestors);
		let file: FileSystemObject = oldParent.files!
			.filter((x: FileSystemObject) => x.name == oldName)[0];
		
		
		// remove entry from old parent path
		oldParent.files = oldParent.files!
			.filter((x: FileSystemObject) => x.name != oldName);		

		
		file.name = newName!;
		let parent = getDir(newAncestors);

		if (parent.files!.filter((x) => x.name == newName).length > 0) {
			oldParent.files.push(file);
			console.error("Writing entity with same path")
			return;
		}

		parent.files!.push(file)

		let oldPath = path;
		path = newPath;
		await invoke("rename_entry", { path, oldPath })

		rerender();
		await handleCancel();
		return;
		
	}

	async function handleRename() {
		// get current file
		let ancestors: string[] = path.split("~~");
		let file = getDir(ancestors);

		ancestors.pop();
		const dlPath = ancestors.join("~~");

		entered_dup_name = get_is_dup_name(dlPath, newName);
		if (entered_dup_name)
			return;

		const oldPath = path;
		file.name = newName;
		ancestors.push(newName);
		path = ancestors.join("~~");


		await invoke("rename_entry", { path, oldPath });


		rerender();
		await handleCancel();
	}

	async function handleDelete() {
		
		deletingStore.update(() => true)
		settingsTrayOpen = false;

		await invoke("delete_entry", { path });

		// get parent of current file
		let ancestors: string[] = path.split("~~");
		let name = ancestors.pop();
		let parent = getDir(ancestors);

		// remove deleted file
		parent.files = parent.files!
			.filter((x: FileSystemObject) => x.name != name);		

		

		// not deleting now
		deletingStore.update(() => false)
		$deletingStore = $deletingStore
		await handleCancel();

	}

	async function handleCancel() {
		settingsTrayOpen = false;
		createFolderTrayOpen = false;
		createDeckTrayOpen = false;
		createDeadlineTrayOpen = false;
		resetDeadlineTrayOpen = false;

		renameTrayOpen = false;
		moveTrayOpen = false;

		newName = "";
		newPath = path.replaceAll("~~", "/");
		deadlineDate = getNextWeekDate();
		deadlineTime = "14:00:00";
		entered_dup_name = false;

		await save_fs();
	}
	

	function focus(el: any){
    	el.focus()
  	}

	function path2name(deck_path: string): string {
		let ancestors = deck_path.split("~~");
		if (ancestors.length > 0)
			return ancestors[ancestors.length - 1];
		else 
			return deck_path;
	}

	async function save_fs() {
			await invoke('write_fs_json', { "fs": $fsStore });
			await invoke('write_global_config', { "config": $configStore });
	}

	function get_is_dup_name(parentPath: string, newName: string): boolean {
		// get children of current folder
		const ancestors: string[] = parentPath.split("~~");
		let children = getDir(ancestors).files!;

		if (children.filter((x: FileSystemObject) => x.name == newName).length > 0) {
			return true;
		}
		return false;
	}


	/**
	 * deadline progress bar
	 */
	// interface PbarData {
	// 	start_date: string,		// MM-DD
	// 	end_date: string,		// MM-DD
	// 	curr_timestamp: number, // epoch timestamp of now minus date deck was created 
	// 	end_timestamp: number,  // epoch timestamp of deadline minus date deck was created 
	// 	days_to_go: number
	// }

	let pbar = {
		start_date: "",
		end_date: "",
		end_time: "",
		curr_timestamp: 0,
		end_timestamp: 0,
		days_to_go: -1
	}

	let progress = -1;
	let deadline_complete: boolean = false
	async function getDeadlineProgress() {
		pbar = await invoke("get_deadline_progress", { "deadlineName": path });
		progress = Math.floor(100 * pbar.curr_timestamp / pbar.end_timestamp);

		if (pbar.curr_timestamp > pbar.end_timestamp)
			deadline_complete = true;

		if (progress > 100)
			progress = 100;
	}
	if (entryType == "deadline")
		getDeadlineProgress();

	function getPbarToolbarText(): string {
		if (!deadline_complete)
			return `${pbar.days_to_go} days until ${pbar.end_date.replace("-", "/")} deadline at ${pbar.end_time}`;
		else
			return `Deadline passed on ${pbar.end_date} at ${pbar.end_time}`
	}

	let resetting_deadline = false;
	async function handleResetDeadline() {
		// deletingstore is a means to reset the home page upon resetting deadline
		deletingStore.update(() => true)
		resetting_deadline = true;

		if (studyIntensity == "")
			studyIntensity = "1";
		
		await invoke("reset_deadline", 
			{ "deadlineName": path, studyIntensity, deadlineDate, deadlineTime });

		await handleCancel()
		await getDeadlineProgress();
		deadline_complete = false;
		resetting_deadline = false;
		deletingStore.update(() => false)
	}

	/**
	 * feature idea: color progress bar based on card completion
	*/
	// function getPbarColor(): string {
	// 	if (quota == null)
	// 		return "";

	// 	if (quota.days_to_go < 0) {
	// 		if (quota.new_left > 0 || quota.review_left > 0)
	// 			return "bg-rose-300";
	// 		else
	// 			return "bg-cyan-400";
	// 	}

	// 	return "bg-columbia"
	// }


</script>



<div class="flex justify-end space-x-4">


	{#if entered_dup_name}
		<!-- warning ! -->
		<div class="float-right">
			<Hint placement="left" text="Duplicate paths not allowed">
				<div class="h-6 w-5 text-blacktext dark:text-columbia font-extrabold">!</div>
			</Hint>
		</div>
	{:else}
		<!-- placeholder -->
		<div class="float-right">
			<div class="h-6 w-5 text-blacktext dark:text-columbia font-extrabold"></div>
		</div>
	
	{/if}

	{#if entryType == "deadline"}


		{#if !resetting_deadline}
			<div class="float-right pt-1 px-2">
				{#if deadline_complete}
					<Hint placement="left" text="Reset deadline">
						<button class="cursor-pointer ring-columbia  focus:outline-none focus:ring duration-75" 
							on:keydown|stopPropagation={() => {
								resetDeadlineTrayOpen = !resetDeadlineTrayOpen;
								settingsTrayOpen = !settingsTrayOpen;
							}}
							on:click|stopPropagation={() => {
								resetDeadlineTrayOpen = !resetDeadlineTrayOpen;
								settingsTrayOpen = !settingsTrayOpen;
							}}
						>
							<svg class="h-4 w-4 mr-2" fill="currentColor" viewBox="0 0 20 20" xmlns="http://www.w3.org/2000/svg" aria-hidden="true">
								<path clip-rule="evenodd" fill-rule="evenodd" d="M15.312 11.424a5.5 5.5 0 01-9.201 2.466l-.312-.311h2.433a.75.75 0 000-1.5H3.989a.75.75 0 00-.75.75v4.242a.75.75 0 001.5 0v-2.43l.31.31a7 7 0 0011.712-3.138.75.75 0 00-1.449-.39zm1.23-3.723a.75.75 0 00.219-.53V2.929a.75.75 0 00-1.5 0V5.36l-.31-.31A7 7 0 003.239 8.188a.75.75 0 101.448.389A5.5 5.5 0 0113.89 6.11l.311.31h-2.432a.75.75 0 000 1.5h4.243a.75.75 0 00.53-.219z"></path>
							</svg>
						</button>
					</Hint>
				{/if}

				{#if pbar.start_date != ""}
					<Hint placement="top" text={getPbarToolbarText()}>
						<!-- shell for progress bar -->
						<div class="h-2 w-20 mb-1 sm:w-32 rounded-md bg-offwhite dark:bg-offblack border">
							<!-- filled-up progress -->
							<div class="h-full rounded-md bg-columbia" style="width: {`${progress}%`}"></div>
						</div>
					</Hint>
				{/if}
			</div>
		{/if}

		<!-- Edit -->
		{#if has_deck}
			<a href={`/${path}/edit`} class="z-10 outline-none">
				<Hint placement="left" text="Edit {path2name(path)}">
					<button
						class="float-right ring-columbia  focus:outline-none focus:ring duration-75 rounded-md">
						<img class="w-6 h-6 p-1 dark:invert" src=pencil.png alt="editing pencil" />
					</button>
				</Hint>
			</a>
		{:else}
			<div class="z-10 outline-none">
				<Hint placement="left" text="Create deck first">
					<button
						class="float-right cursor-default ring-columbia rounded-md focus:outline-none focus:ring duration-75">
						<img class="w-6 h-6 p-1 dark:invert opacity-30" src=pencil.png alt="editing pencil" />
					</button>
				</Hint>
			</div>

		{/if}

		<!-- Review -->
		{#if has_deck && quota != null && (quota.new_left > 0 || quota.review_left > 0)}
			<a href={`/${path}/review`} class="z-20 pr-2 outline-none">
				<Hint placement="left" text="Review {deadline_complete ? "past deadline" : `cards in ${path2name(path)}`}">
					<button class="float-right z-30 ring-columbia  focus:outline-none focus:ring duration-75 rounded-md">
						<!-- deck from <a href="https://www.flaticon.com/free-icons/flash-cards" title="flash cards icons">Flash cards icons created by manshagraphics - Flaticon</a> */ -->
						<img class="w-6 h-6 p-1 dark:invert" src=flash-cards.png alt="review" />
					</button>
				</Hint>
			</a>
			{:else}
			<div class="z-20 pr-2 outline-none">
				<Hint placement="left" text="No cards to review">
					<button
						class="float-right cursor-default ring-columbia rounded-md focus:outline-none focus:ring duration-75">
						<img class="w-6 h-6 p-1 dark:invert opacity-40" src=flash-cards.png alt="review" />
					</button>
				</Hint>
			</div>
			
		{/if}


	{/if}


	{#if quota != null} 
		<!-- <div class="float-right -z-10">
			<Hint placement="left" text="{quota.num_progressed} {quota.num_progressed == 1 ? "card" : "cards"} progressed today">
				<div class="h-6 w-5 text-blacktext dark:text-columbia font-serif">{quota.num_progressed}</div>
			</Hint>
		</div> -->
		<div class="float-right z-20">
			<Hint placement="left" text="{quota.num_progressed} {quota.num_progressed == 1 ? "card" : "cards"} advanced today">
				<div class="h-6 w-5 text-blacktext dark:text-columbia font-serif">{quota.num_progressed} </div>
			</Hint>
		</div>

		<div class="float-right z-30">
			<Hint placement="left" text="{quota.new_left} new {quota.new_left == 1 ? "card" : "cards"} to practice today">
				<div class="h-6 w-5 text-blacktext dark:text-columbia font-serif">{quota.new_left} </div>
			</Hint>
		</div>

		<div class="float-right z-40">
			<Hint placement="left" text="{quota.review_left} reviewed {quota.new_left == 1 ? "card" : "cards"} to practice today">
				<div class="h-6 w-4 text-blacktext dark:text-columbia font-serif">{quota.review_left} </div>
			</Hint>
		</div>
	{/if}
		


	<!-- Dropend -->
	<div class="z-40">
		<Hint placement="left" text="Create or modify files">
			<button
					class="float-right ring-columbia focus:outline-none focus:ring duration-75 rounded-md -ring-offset-4"
					on:click|stopPropagation={toggleSettingsTray}
				>
					<img class="h-6 w-6 p-1 mb-1 mb-r dark:invert" src=settings-gear-black.png alt="setting gear" />
			</button>
		</Hint>
	</div>
	
		

	{#if settingsTrayOpen}
	<div
		class="absolute flex justify-between z-50 divide-y divide-gray-100 rounded-lg {!actionTrayOpen ? "w-28" : "w-64"} bg-white dark:bg-slate-700 text-blacktext dark:text-whitetext"
		use:clickOutside 
		on:click_outside={handleClickOutside}>

		{#if !actionTrayOpen}
		<!-- error on `click_outside` is due to svelte; like a necessary deprecation error, ignore it -->

		<!-- Dropdown menu -->
		<ul class="px-1 py-2 text-sm ml-3 border-r-[1px] border-columbia -border-spacing-4 rounded-lg" aria-labelledby="dropdownRightEndButton">
			{#if entryType === "folder"}
			<li>
				<div role="button" 
					on:click={() => { createFolderTrayOpen = true; }} on:keypress={() => { createFolderTrayOpen = true; }}
					class="hover:bg-columbia border-x-2 dark:hover:bg-columbia-dark rounded-lg border-columbia block px-4 py-2 dark:hover:text-whitetext">
					Create Folder
				</div>
			</li>
			<li>
				<div role="button"
					on:click={() => { createDeadlineTrayOpen = true; }} on:keypress={() => { createDeadlineTrayOpen = true; }}
					class="hover:bg-columbia border-x-2 dark:hover:bg-columbia-dark rounded-lg block border-columbia px-4 py-2 dark:hover:text-white">
					Create Deadline
				</div>
			</li>
			{/if}

			{#if entryType === "deadline" && !deadline_complete}
			<li>
				<div role="button" 
					on:click={() => { createDeckTrayOpen = true; }} on:keypress={() => { createDeckTrayOpen = true; }}
					class="hover:bg-columbia border-x-2 dark:hover:bg-columbia-dark rounded-lg  block px-4 py-2 border-columbia dark:hover:text-white">
					Create Deck
				</div>
			</li>
			{/if}

			<li>
				<div role="button" 
					on:click={() => { renameTrayOpen = true; }} on:keypress={() => { renameTrayOpen = true; }}
					class="hover:bg-columbia border-x-2 dark:hover:bg-columbia-dark  rounded-lg block px-4 py-2 border-columbia dark:hover:text-white">
					Rename
				</div>
			</li>

			<!-- {#if entryType !== "deck" && path.includes("~~")}
			<li>
				<div role="button" 
					on:click={() => { moveTrayOpen = true; }} on:keypress={() => { moveTrayOpen = true; }}
					class="hover:bg-columbia border-x-2 dark:hover:bg-columbia-dark rounded-lg block px-4 py-2 border-columbia dark:hover:text-white">
					Move
				</div>
			</li>
			{/if} -->

			{#if path.includes("~~")}
			<li>
				<div role="button" 
					on:click={handleDelete } on:keypress={ handleDelete }
					class="hover:bg-columbia border-x-2 dark:hover:bg-columbia-dark rounded-lg  block px-4 py-2 border-columbia dark:hover:text-white">
					Delete
				</div>
			</li>
			{/if}
		</ul>


		<!-- open deadline tray -->
		{:else}

			<form class="w-full m-5 py-1"
					on:submit={() => {
						settingsTrayOpen = false;
						if (renameTrayOpen) handleRename() 
						else if (moveTrayOpen) handleMove()
						else if (resetDeadlineTrayOpen) handleResetDeadline()
						else handleCreateEntry()
					}}>
				<div class="border-b border-columbia py-2 grid {createDeadlineTrayOpen ? "grid-rows-4" : "grid-rows-2" } grid-cols-3 gap-2">
					<!-- name for Create -->
					{#if !moveTrayOpen && !renameTrayOpen && !resetDeadlineTrayOpen}
						<input type="text" use:focus placeholder="Enter Name" bind:value={newName} class="h-8 col-span-3 hover:bg-columbia dark:hover:bg-columbia-dark dark:bg-offblack border-2 border-columbia rounded-lg block px-4 dark:hover:text-whitetext ring-columbia focus:outline-none focus:ring duration-75"/>
					<!-- Rename -->
					{:else if !moveTrayOpen && renameTrayOpen && !resetDeadlineTrayOpen} 
						<input type="text" use:focus placeholder="Enter Name" bind:value={newName} class="h-8 col-span-3 hover:bg-columbia dark:hover:bg-columbia-dark dark:bg-offblack border-2 border-columbia rounded-lg block px-4 dark:hover:text-whitetext ring-columbia focus:outline-none focus:ring duration-75"/>
					<!-- Move -->
					{:else if !resetDeadlineTrayOpen}
						<input bind:value={newPath} use:focus required class="h-8 col-span-3 hover:bg-columbia dark:hover:bg-columbia-dark dark:bg-offblack border-2 border-columbia rounded-lg block px-4 dark:hover:text-whitetext"/>
					{/if}
					
					{#if createDeadlineTrayOpen || resetDeadlineTrayOpen}
						<input type="date" id="deadlineDate" bind:value={deadlineDate} required class="h-8 col-span-3 hover:bg-columbia dark:hover:bg-columbia-dark dark:bg-offblack border-2 border-columbia rounded-lg block px-4 dark:hover:text-whitetext ring-columbia  focus:outline-none focus:ring duration-75"/>
						<input type="time" bind:value={deadlineTime} required class="h-8 col-span-3 hover:bg-columbia dark:hover:bg-columbia-dark dark:bg-offblack border-2 border-columbia rounded-lg block px-4 dark:hover:text-whitetext ring-columbia  focus:outline-none focus:ring duration-75"/>
						<div class="justify-center h-8 col-span-3 hover:bg-columbia rounded-lg dark:hover:bg-columbia block dark:hover:text-whitetext">
							
							<select 
								bind:value={studyIntensity}
								placeholder="Study Intensity"
								class="
									form-select 
									block appearance-none
									w-full
									h-full
									text-base
									px-4
									hover:bg-columbia dark:hover:bg-columbia-dark
									border-2 border-columbia 
									
									font-normal
									rounded-lg
									transition
									ease-in-out
									m-0
									dark:bg-offblack dark:text-offwhite
									focus:text-gray-700 focus:bg-white ring-columbia  focus:outline-none focus:ring duration-75" 
									aria-label="Default select example">
								<option  value="" selected disabled>Study Intensity</option>
								
								<option value="1">Low</option>
								<option value="2">Normal</option>
								<option value="3">High</option>
								{#if resetDeadlineTrayOpen}
									<option value="0">Unfinished only</option>
								{/if}
							</select>
						</div>
					{/if}

					
					<button 
						type="submit" class="h-8 col-span-2 text-sm hover:bg-columbia dark:hover:bg-columbia-dark dark:bg-offblack border-2 border-columbia rounded-lg block px-4 dark:hover:text-whitetext ring-columbia  focus:outline-none focus:ring duration-75">
						{renameTrayOpen ? "Rename" : resetDeadlineTrayOpen ? "Reset Deadline" : "Create"}
					</button>
					<button 
						type="button" on:click={ handleCancel	 } 
						class="h-8 col-span-1 hover:bg-columbia border-2 border-columbia dark:bg-offblack text-sm dark:hover:bg-columbia-dark rounded-lg block dark:hover:text-whitetext ring-columbia  focus:outline-none focus:ring duration-75">
						Cancel
					</button>
				</div>
			</form>

		{/if}
	</div>
	{/if}

</div>


