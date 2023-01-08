<script lang="ts">
	import File from './File.svelte';
	// import KaTeXRenderer from './KaTeXRenderer.svelte';
	import settingsGear from '$lib/images/settings-gear-black.png';
	import { clickOutside } from './actions/click_outside';

	type FileSystemObject = {
		type: 'folder' | 'file';
		name: string;
		files?: FileSystemObject[];
		expanded?: boolean;
		deadlineDate?: string;
		deadlineTime?: string;
	};

	export let expanded = false;
	export let name: string = '';
	export let type: 'folder' | 'file' = 'folder';
	export let deadlineDate: string | null = null;
	export let deadlineTime: string | null = null;
	export let files: FileSystemObject[];
	let foldersMuted: boolean = false;

	function handleNewFolder() {
		// add a new folder
	}

	function handleNewFile() {
		//
	}

	function toggle() {
		if (!foldersMuted) {
			expanded = !expanded;
		}
	}

	let settingsTrayOpen: boolean = false;
	function toggleSettingsTray() {
		settingsTrayOpen = !settingsTrayOpen;
	}

	let deadlineEditable: boolean = false;
	let nameEditable: boolean = false;

	// function handleDeadlineClick() {
	// 	deadlineEditable = true;
	// 	foldersMuted = true;
	// }

	// Write out click logic for the deadline string
	function handleOutclick() {
		nameEditable = false;
		deadlineEditable = false;
		foldersMuted = false;
	}

	function focusOnMount(el: any) {
		el.focus();
	}

	function handleEditableDeadlineKeypress(e: KeyboardEvent) {
		if (e.code === 'Enter') {
			deadlineEditable = false;
			foldersMuted = false;
		}
	}

	function handleEditableNameKeypress(e: KeyboardEvent) {
		if (e.code === 'Enter') {
			nameEditable = false;
			foldersMuted = false;
		}
	}

	// Deadline Should Also Save on ENTER
</script>

<div class="folder-container">
	<span class="folder" class:expanded on:click={toggle} on:keypress={toggle}>
		<span class="span-left">
			{#if nameEditable}
				<input
					type="text"
					bind:value={name}
					use:focusOnMount
					on:click|stopPropagation
					on:keypress|stopPropagation={handleEditableNameKeypress}
					use:clickOutside={nameEditable}
					on:outclick={handleOutclick}
				/>
			{:else}
				{name}
			{/if}
		</span>
		<span class="span-right">
			{#if deadlineEditable}
				<span
					on:click|stopPropagation
					on:keypress|stopPropagation={handleEditableDeadlineKeypress}
					use:clickOutside={deadlineEditable}
					on:outclick={handleOutclick}
				>
					<!-- <input type="text" bind:value={deadlineString} use:focusOnMount /> -->
					<input type="date" bind:value={deadlineDate} />
					<input type="time" bind:value={deadlineTime} />
				</span>
			{:else}
				<span>{deadlineDate && deadlineTime ? `Due ${deadlineDate} at ${deadlineTime}` : ''}</span>
			{/if}
			<span class="folder-buttons-span">
				<!-- Will eventually wrap the review button in an a tag and move the on:click|stopPro.. to that tag -->
				<button on:click|stopPropagation>Review</button>
				<button
					class="folder-settings-button"
					on:keydown|stopPropagation={toggleSettingsTray}
					on:click|stopPropagation={toggleSettingsTray}
				>
					<img class="gear" src={settingsGear} alt="setting gear" />
				</button></span
			>
		</span>
	</span>
	{#if settingsTrayOpen}
		<div
			class="settings-tray"
			use:clickOutside={settingsTrayOpen}
			on:outclick={() => {
				settingsTrayOpen = false;
			}}
		>
			<ul>
				<li class="settings-tray-option">
					<button
						on:click={() => {
							deadlineEditable = true;
							foldersMuted = true;
							settingsTrayOpen = false;
						}}
					>
						Edit Deadline
					</button>
				</li>
				<li class="settings-tray-option">
					<button
						on:click={() => {
							nameEditable = true;
							foldersMuted = true;
							settingsTrayOpen = false;
						}}
					>
						Edit Deck Name
					</button>
				</li>
				<li class="settings-tray-option">Edit Deck</li>
				<li class="settings-tray-option">Delete Deck</li>
			</ul>
		</div>
	{/if}
</div>

{#if expanded}
	<ul>
		{#each files as file, i}
			<li>
				{#if file.type === 'folder'}
					<svelte:self
						bind:files={file.files}
						bind:name={file.name}
						bind:expanded={file.expanded}
						bind:deadlineDate={file.deadlineDate}
						bind:deadlineTime={file.deadlineTime}
						type="folder"
					/>
				{:else}
					<File {...file} />
				{/if}
			</li>
		{/each}
		<li>
			{#if type === 'folder'}
				<span class="create-file-folder-row">
					<span class="span-left"><button on:click={handleNewFile}>+ File</button></span>
					<span class="span-right" />
				</span>
				<span class="create-file-folder-row">
					<span class="span-left"><button on:click={handleNewFolder}>+ Folder</button></span>
					<span class="span-right" />
				</span>
			{/if}
		</li>
	</ul>
{/if}

<style>
	.folder-buttons-span {
		width: 100px;
		display: flex;
		justify-content: space-between;
	}

	.folder-container {
		position: relative;
	}

	.folder {
		display: flex;
		justify-content: space-between;
		align-items: center;
		width: 100%;
		/* padding: 2px 2px 2px 1.5em; */
		/* background: url(/folder.svg) 0 0.1em no-repeat; */
		padding: 2px 10px;
		background: 0 0.1em no-repeat;
		background-size: 1em 1em;
		font-weight: bold;
		cursor: pointer;
		margin: 1px;
		border-radius: 2px;
	}

	.folder:hover {
		background-color: #eee;
	}

	.create-file-folder-row {
		display: flex;
		justify-content: space-between;
		align-items: center;
		width: 100%;
		/* padding: 2px 2px 2px 1.5em; */
		/* background: url(/folder.svg) 0 0.1em no-repeat; */
		padding: 2px 10px;
		background: 0 0.1em no-repeat;
		background-size: 1em 1em;
		font-weight: bold;
		cursor: pointer;
		margin: 1px;
		border-radius: 2px;
	}

	.create-file-folder-row:hover {
		background-color: rgb(245, 245, 245);
	}

	.span-right {
		display: flex;
		justify-content: space-between;
		width: 300px;
	}
	.span-left {
		width: 33%;
	}

	/* .expanded {
		background-image: url(/folder-open.svg);
	} */

	ul {
		padding: 0.2em 0 0 0.5em;
		margin: 0 0 0 0.5em;
		list-style: none;
		border-left: 1px solid #eee;
	}

	li {
		padding: 0.2em 0;
	}

	.gear {
		height: 1em;
		width: 1em;
	}

	.gear:hover {
		height: 1.2em;
		width: 1.2em;
	}

	.folder-settings-button {
		height: 100%;
		width: 2em;
		border: none;
		display: flex;
		justify-content: center;
		align-items: center;
		background-color: transparent;
		border-radius: 100px;
	}

	.settings-tray {
		padding: 10px;
		display: flex;
		align-items: flex-start;
		justify-content: center;
		position: absolute;
		right: 0;
		top: 2em;
		width: 150px;
		background-color: #eee;
		box-shadow: 0px 0px 10px #eee;
		border-radius: 2px;
		z-index: 4;
	}

	.settings-tray ul {
		padding: 0;
		margin: 0;
		font-size: 0.8em;
	}

	.settings-tray li {
		border: none;
		padding: none;
	}
</style>
