
<script lang="ts">
	import {clickOutside} from '$lib/actions/click_outside.js';
    import { invoke } from '@tauri-apps/api/tauri';
	import Hint from 'svelte-hint';
	import type { EntryData } from '$lib/stores/folderSystemStore'
	import folderSystemStore from '$lib/stores/folderSystemStore'
	import rootFolderStore from '$lib/stores/rootFolderStore'
	import reloadStore from '$lib/stores/reloadStore'
	import { fade } from "svelte/transition";


	export let entryData: EntryData;


	// click on gear
	export let settingsTrayOpen = false;
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

		resetDeadlineTrayOpen = false;
	}

	// specialty trays
	let newName: string = "";
	let renameName: string = entryData.entry_name;
	let studyIntensity = "";

	let createFolderTrayOpen = false;
	let createDeckTrayOpen = false;
	let createDeadlineTrayOpen = false;
	let createAnkiTrayOpen = false;
	let resetDeadlineTrayOpen = false;

	let renameTrayOpen = false;
	let moveTrayOpen = false;

	$: actionTrayOpen = createFolderTrayOpen || createDeadlineTrayOpen || 
		createDeckTrayOpen || renameTrayOpen || moveTrayOpen || resetDeadlineTrayOpen || createAnkiTrayOpen;

	let deadlineDate: string | null = getNextWeekDate();
	let deadlineTime: string | null = "14:00";
	let entered_dup_name = false;
	let new_per_day = "8";


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

	interface EntryMetadata {
		entry_type: string,
		deadline_date: string | null,
		study_intensity: number | null,
		new_per_day: number
	}

	// settings tray buttons
	let entered_past_deadline = false;
	async function handleCreateEntry() {
		if (newName.length == 0)
			return;
			
		let parentId = entryData.entry_id;
		if (parentId) {
			entered_dup_name = await getIsDupName(parentId, newName);
			if (entered_dup_name) return;
		}		

		// specify what type of entry is being created
		let newType: "folder" | "deadline" | "deck" | "ankibox";

		let new_deadline_date;
		let intensity;

		if (createFolderTrayOpen) {
			newType = "folder";
			new_deadline_date = null;
			intensity = null
		} else if (createDeadlineTrayOpen) {
			newType = "deadline";
			new_deadline_date = deadlineDate + " " + deadlineTime + ":00";
			intensity = studyIntensity == "" ? 2 : parseInt(studyIntensity);
		} else if (createAnkiTrayOpen) {
			newType = "ankibox";
			new_deadline_date = null;
			intensity = null
		} else {
			newType = "deck";
			new_deadline_date = null;
			intensity = null;
		}

		// create_entry(conn: &mut PgConnection, entry_name: &str, parent_id: Option<i32>, md: EntryMetadata)
		let md: EntryMetadata = {
			entry_type: newType,
			deadline_date: new_deadline_date,
			study_intensity: intensity,
			new_per_day: parseInt(new_per_day)
		}
		newName = newName.slice(0, 29);
		invoke("create_entry", { "entryName": newName, parentId, md});

		reloadStore.update((state) => !state)
		handleCancel();
	}

	async function handleRename() {
		
		let parentId = getParentId();
		if (!parentId) return;

		renameName = renameName.slice(0, 29);
		entered_dup_name = await getIsDupName(parentId, renameName);
		if (entered_dup_name)
			return;

		await invoke("rename_entry", { "entryId": entryData.entry_id, renameName });

		handleCancel();
		reloadStore.update((state) => !state)
	}

	async function handleDelete() {
		settingsTrayOpen = false;

		await invoke("delete_entry", { "entryId": entryData.entry_id });

		reloadStore.update((state) => !state)
		handleCancel();

	}

	async function handleCancel() {
		settingsTrayOpen = false;
		createFolderTrayOpen = false;
		createDeckTrayOpen = false;
		createDeadlineTrayOpen = false;
		createAnkiTrayOpen = false;
		resetDeadlineTrayOpen = false;

		renameTrayOpen = false;
		moveTrayOpen = false;

		newName = "";
		deadlineDate = getNextWeekDate();
		deadlineTime = "14:00";
		entered_dup_name = false;
		entered_past_deadline = false;
	}
	

	function focus(el: any){
    	el.focus()
  	}

	async function getIsDupName(parentId: number, newName: string): Promise<boolean> {
		let dup: boolean = await invoke("is_duplicate_name", {parentId, newName});
		return dup;
	}
	
	function getParentId(): number | null {
		let parentId;
		for (let pair of $folderSystemStore.pairs) {
			if (pair.child_id == entryData.entry_id) {
				parentId = pair.parent_id;
				break;
			}
		}

		if (!parentId) 
			return null;

		return parentId;
	}


	let deadline_complete: boolean;
	let entry_deadline_date: string;
	async function getDeadlineDate() {
		[entry_deadline_date, deadline_complete] = await invoke("get_deadline_date", { "deadlineId": entryData.entry_id });
	}
	if (entryData.entry_type == "deadline")
		getDeadlineDate();

	async function handleResetDeadline() {
		let intensity = studyIntensity == "" ? 2 : parseInt(studyIntensity);

		let newDeadlineDate = deadlineDate + " " + deadlineTime + ":00";
		entered_past_deadline = await invoke("entered_past_deadline", { "deadline": newDeadlineDate })
		if (entered_dup_name) return
		
		await invoke("reset_deadline", 
			{ "deadlineId": entryData.entry_id, "studyIntensity": intensity, newDeadlineDate });

		reloadStore.update((state) => !state);
		handleCancel();
	}


	let newParentId: number = $folderSystemStore.data[0].entry_id;
	async function handleMove() {
		entered_dup_name = await getIsDupName(newParentId, entryData.entry_name);
		if (entered_dup_name)
			return;

		await invoke("move_entry", { "entryId": entryData.entry_id, newParentId });
		reloadStore.update((state) => !state)
	}


</script>



<div class="flex justify-end space-x-3 lg:space-x-5">


	<!-- warning mark in case user performs an invalid action -->
	{#if entered_dup_name}
		<!-- warning sign ! -->
		<div class="float-right">
			<Hint placement="top" text="Duplicate paths not allowed">
				<div class="h-6 w-5 text-blacktext dark:text-columbia font-extrabold">!</div>
			</Hint>
		</div>
	{:else if entered_past_deadline}
		<div class="float-right">
			<Hint placement="top" text="Deadline must be set in the future">
				<div class="h-6 w-5 text-blacktext dark:text-columbia font-extrabold">!</div>
			</Hint>
		</div>
	{:else}
		<!-- placeholder for spacing -->
		<div class="float-right">
			<div class="h-6 w-5 text-blacktext dark:text-columbia font-extrabold"></div>
		</div>
	
	{/if}

	{#if entryData.entry_type == "deadline" || entryData.entry_type == "ankibox"}


		<div class="float-right pt-1 px-2 flex flex-row">
			
			{#if deadline_complete}
				<Hint placement="top" text="Reset deadline">
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
						<svg class="h-4 w-4 mb-1 mr-6" fill="currentColor" viewBox="0 0 20 20" xmlns="http://www.w3.org/2000/svg" aria-hidden="true">
							<path clip-rule="evenodd" fill-rule="evenodd" d="M15.312 11.424a5.5 5.5 0 01-9.201 2.466l-.312-.311h2.433a.75.75 0 000-1.5H3.989a.75.75 0 00-.75.75v4.242a.75.75 0 001.5 0v-2.43l.31.31a7 7 0 0011.712-3.138.75.75 0 00-1.449-.39zm1.23-3.723a.75.75 0 00.219-.53V2.929a.75.75 0 00-1.5 0V5.36l-.31-.31A7 7 0 003.239 8.188a.75.75 0 101.448.389A5.5 5.5 0 0113.89 6.11l.311.31h-2.432a.75.75 0 000 1.5h4.243a.75.75 0 00.53-.219z"></path>
						</svg>
					</button>
				</Hint>
			{/if}
			{#if entry_deadline_date}
				<Hint placement="top" text="Deadline date">
				<div class=" z-10 mb-8">
						<div class="w-22 { !deadline_complete ? "text-darktext dark:text-columbia" : "opacity-30" } font-mono">
							<p class="align-top text-sm leading-4">{entry_deadline_date}</p>
						</div>
				</div>
				</Hint>

			{/if}
		</div>
			<!-- Edit -->
			{#if !deadline_complete}
				<a href={`/${entryData.entry_id}/edit`} class="z-10 outline-none">
					<Hint placement="top" text="Edit">
						<button class="float-right ring-columbia -mr-1  focus:outline-none focus:ring duration-75 rounded-md">
							<img class="w-6 h-6 p-1 dark:invert" src=pencil.png alt="editing pencil" />
						</button>
					</Hint>
				</a>
			{:else}
				<div class="z-20 outline-none">
					<Hint placement="top" text="Reset before editing">
						<button class="float-right cursor-default -mr-1 ring-columbia rounded-md focus:outline-none focus:ring duration-75">
							<img class="w-6 h-6 p-1 dark:invert opacity-30" src=pencil.png alt="editing pencil" />
						</button>
					</Hint>
				</div>
			{/if}
	
			<!-- Review -->
			{#if entryData.entry_quota != null && (entryData.entry_quota.new_left > 0 || entryData.entry_quota.review_left > 0)}
				<a href={`/${entryData.entry_id}/review`} class="z-20 outline-none">
					<Hint placement="top" text="Review">
						<button class="float-right z-30 ring-columbia -ml-1 focus:outline-none focus:ring duration-75 rounded-md">
							<img class="w-6 h-6 p-1 dark:invert" src=flash-cards.png alt="review" />
						</button>
					</Hint>
				</a>
			{:else}
				<div class="z-20 outline-none">
					<Hint placement="top" text="No cards to review">
						<button
							class="float-right cursor-default ring-columbia -ml-1 rounded-md focus:outline-none focus:ring duration-75">
							<img class="w-6 h-6 p-1 dark:invert opacity-40" src=flash-cards.png alt="review" />
						</button>
					</Hint>
				</div>
			{/if}

	{/if}


	{#if entryData.entry_quota != null && entryData.entry_type != 'folder'} 
	

		<div class=" space-x-0">
	  
			<div class="float-right z-40">
				<Hint placement="top" text="{entryData.entry_quota.review_left} review">
					<div class="h-6 w-7 -mr-1 text-blacktext dark:text-columbia font-serif">{entryData.entry_quota.review_left}</div>
				</Hint>
			</div>


			<div class="float-right z-30">
				<Hint placement="top" text="{entryData.entry_quota.new_left} new">
					<div class="h-6 w-7 text-blacktext dark:text-columbia font-serif">{entryData.entry_quota.new_left}</div>
				</Hint>
			</div>

			<div class="float-right">
				<Hint placement="top" text="{entryData.entry_quota.num_progressed} practiced">
					<div class="h-6 w-7 ml-2 text-blacktext dark:text-columbia font-serif">{entryData.entry_quota.num_progressed} </div>
				</Hint>
			</div>

		</div>
	{:else if entryData.entry_type=="ankibox" || entryData.entry_type=="quota"}
		<div class=" space-x-0 ">
	  
			<div class="float-right z-40 ">
				<Hint placement="top" text="0 review">
					<div class="h-6 w-7 -mr-1 text-blacktext dark:text-columbia font-serif">0</div>
				</Hint>
			</div>

			<div class="float-right z-30">
				<Hint placement="top" text="0 new">
					<div class="h-6 w-7 text-blacktext dark:text-columbia font-serif">0</div>
				</Hint>
			</div>

			<div class="float-right">
				<Hint placement="top" text="0 practiced">
					<div class="h-6 w-7 ml-2 text-blacktext dark:text-columbia font-serif">0</div>
				</Hint>
			</div>

		</div>

	{/if}
		


	<!-- vertical ellipsis to open settings tray -->
	<div class="z-40">
		<Hint placement="top" text="Actions">
			<button class="float-right ring-columbia focus:outline-none focus:ring duration-75 rounded-md -ring-offset-4" on:click|stopPropagation={toggleSettingsTray}>
					<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="currentColor" class="w-6 h-6">
						<path fill-rule="evenodd" d="M10.5 6a1.5 1.5 0 113 0 1.5 1.5 0 01-3 0zm0 6a1.5 1.5 0 113 0 1.5 1.5 0 01-3 0zm0 6a1.5 1.5 0 113 0 1.5 1.5 0 01-3 0z" clip-rule="evenodd" />
					</svg>
			</button>
		</Hint>
	</div>
	
		

	{#if settingsTrayOpen}
	<div
		in:fade="{{ duration: 150 }}" out:fade="{{ duration: 150 }}"
		class="absolute flex justify-between z-50 divide-y divide-gray-100 rounded-lg {!actionTrayOpen ? "w-28" : "w-64"} bg-white dark:bg-slate-700 text-blacktext dark:text-whitetext"
		use:clickOutside on:click_outside|capture={handleClickOutside}> <!-- error on `click_outside` is due to svelte; like a necessary deprecation error, ignore it -->

		{#if !actionTrayOpen}

		<!-- Dropdown menu -->
		<ul class="px-1 py-2 text-sm ml-3 border-r-[1px] border-columbia -border-spacing-4 rounded-lg" aria-labelledby="dropdownRightEndButton">
			{#if entryData.entry_type === "folder" }
				<li>
					<div role="button" tabindex="0"
						on:click={() => { createFolderTrayOpen = true; }} on:keypress={() => { createFolderTrayOpen = true; }}
						class="hover:bg-columbia border-x-2 dark:hover:bg-columbia-dark rounded-lg border-columbia block px-4 py-2 dark:hover:text-whitetext">
						Create Folder
					</div>
				</li>
				<!-- <li>
					<div role="button" tabindex="0"
						on:click={() => { createDeadlineTrayOpen = true; }} on:keypress={() => { createDeadlineTrayOpen = true; }}
						class="hover:bg-columbia border-x-2 dark:hover:bg-columbia-dark rounded-lg block border-columbia px-4 py-2 dark:hover:text-white">
						Create Deadline
					</div>
				</li> -->
				<li>
					<div role="button" tabindex="0"
						on:click={() => { createAnkiTrayOpen = true; }} on:keypress={() => { createDeadlineTrayOpen = true; }}
						class="hover:bg-columbia border-x-2 dark:hover:bg-columbia-dark rounded-lg block border-columbia px-4 py-2 dark:hover:text-white">
						Create Subject
					</div>
				</li>
			{/if}

			{#if entryData.entry_type === "deadline" && !deadline_complete || entryData.entry_type === "ankibox"}
				<li>
					<div role="button" tabindex="0"
						on:click={() => { createDeckTrayOpen = true; }} on:keypress={() => { createDeckTrayOpen = true; }}
						class="hover:bg-columbia border-x-2 dark:hover:bg-columbia-dark rounded-lg  block px-4 py-2 border-columbia dark:hover:text-white">
						Create Deck
					</div>
				</li>
			{/if}

			<li>
				<div role="button" tabindex="0"
					on:click={() => { renameTrayOpen = true; }} on:keypress={() => { renameTrayOpen = true; }}
					class="hover:bg-columbia border-x-2 dark:hover:bg-columbia-dark  rounded-lg block px-4 py-2 border-columbia dark:hover:text-white">
					Rename
				</div>
			</li>

			{#if entryData.entry_type !== "deck" && !$rootFolderStore.includes(entryData.entry_id)}
				<li>
					<div role="button" tabindex="0"
						on:click={() => { moveTrayOpen = true; }} on:keypress={() => { moveTrayOpen = true; }}
						class="hover:bg-columbia border-x-2 dark:hover:bg-columbia-dark rounded-lg block px-4 py-2 border-columbia dark:hover:text-white">
						Move
					</div>
				</li>
			{/if}

			{#if !$rootFolderStore.includes(entryData.entry_id)}
				<li>
					<div role="button" tabindex="0"
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
						<input type="text" use:focus placeholder="Enter Name" bind:value={newName} class="h-8 {createDeckTrayOpen ? "col-span-2" : "col-span-3"} hover:bg-columbia dark:hover:bg-columbia-dark dark:bg-offblack border-2 border-columbia rounded-lg block px-4 dark:hover:text-whitetext ring-columbia focus:outline-none focus:ring duration-75"/>

						 {#if createDeckTrayOpen}
							<Hint placement="top" text="Set New Cards Per Day">
								<input bind:value={new_per_day} type="number" id="quantity" name="quantity" min="1" max="100" step="1"
									class="h-8 col-span-1 appearance-none hover:bg-columbia dark:hover:bg-columbia-dark dark:bg-offblack border-2 border-columbia rounded-lg block px-3 dark:hover:text-whitetext ring-columbia focus:outline-none focus:ring-2 duration-75">
							</Hint>
						{/if}
		
					<!-- Rename -->
					{:else if !moveTrayOpen && renameTrayOpen && !resetDeadlineTrayOpen} 
						<input type="text" use:focus placeholder="Enter Name" bind:value={renameName} class="h-8 col-span-3 hover:bg-columbia dark:hover:bg-columbia-dark dark:bg-offblack border-2 border-columbia rounded-lg block px-4 dark:hover:text-whitetext ring-columbia focus:outline-none focus:ring duration-75"/>
					<!-- Move -->
					{:else if !resetDeadlineTrayOpen}
						<!-- <label for="folders" class="w-32 -mb-8">Choose Location:</label> -->
						
						<select id="folders" bind:value={newParentId} class="h-8 col-span-3 appearance-none hover:bg-columbia dark:hover:bg-columbia-dark dark:bg-offblack border-2 border-columbia rounded-lg block px-4 dark:hover:text-whitetext ring-columbia  focus:outline-none focus:ring duration-75">
							{#each $folderSystemStore.data as entry}
								{#if entry.entry_type == "folder"}
									<option value={entry.entry_id}>{entry.entry_name}</option>
								{/if}
							{/each}
						</select>
					{/if}
					
					{#if createDeadlineTrayOpen || resetDeadlineTrayOpen}
						<input type="date" id="deadlineDate" bind:value={deadlineDate} required class="h-8 col-span-3 hover:bg-columbia dark:hover:bg-columbia-dark dark:bg-offblack border-2 border-columbia rounded-lg block px-4 dark:hover:text-whitetext ring-columbia  focus:outline-none focus:ring duration-75"/>
						<input type="time" bind:value={deadlineTime} required class="h-8 col-span-3 hover:bg-columbia dark:hover:bg-columbia-dark dark:bg-offblack border-2 border-columbia rounded-lg block px-4 dark:hover:text-whitetext ring-columbia  focus:outline-none focus:ring duration-75"/>
						<div class="justify-center h-8 col-span-3 hover:bg-columbia rounded-lg dark:hover:bg-columbia block dark:hover:text-whitetext">
							
							<select 
								bind:value={studyIntensity}
								placeholder="Study Intensity"
								class="form-select block appearance-none w-full h-full text-base px-4 hover:bg-columbia dark:hover:bg-columbia-dark border-2 border-columbia font-normal rounded-lg transition ease-in-out m-0 dark:bg-offblack dark:text-offwhite focus:text-gray-700 focus:bg-white ring-columbia  focus:outline-none focus:ring duration-75" 
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
						{renameTrayOpen ? "Rename" : resetDeadlineTrayOpen ? "Reset Deadline" : moveTrayOpen ? "Choose Folder" : "Create"}
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


