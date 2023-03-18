

			<div class="flex flex-row absolute bottom-0 left-4"	>
				


				<!-- backward chevron -->
				<!-- <div class="z-30 float-left">
					{#if num_forward > 0}
						<Hint placement="top" text="Press < to go back">
							<button
								on:click={getLastCard}
								class="float-right cursor-pointer ring-columbia focus:outline-none focus:ring duration-75">
								<svg class="dark:invert flex-none h-7 w-7 cursor-pointer  "
									fill="currentColor" viewBox="0 0 20 20" xmlns="http://www.w3.org/2000/svg" aria-hidden="true" data-darkreader-inline-fill="">
									<path clip-rule="evenodd" fill-rule="evenodd" d="M12.79 5.23a.75.75 0 01-.02 1.06L8.832 10l3.938 3.71a.75.75 0 11-1.04 1.08l-4.5-4.25a.75.75 0 010-1.08l4.5-4.25a.75.75 0 011.06.02z"></path>
								</svg>
							</button>
						</Hint>
					{:else}
						<Hint placement="top" text="Press < to go back">
							<div class="float-right cursor-default opacity-50">
								<svg class="dark:invert flex-none h-7 w-7 "
									fill="currentColor" viewBox="0 0 20 20" xmlns="http://www.w3.org/2000/svg" aria-hidden="true" data-darkreader-inline-fill="">
									<path clip-rule="evenodd" fill-rule="evenodd" d="M12.79 5.23a.75.75 0 01-.02 1.06L8.832 10l3.938 3.71a.75.75 0 11-1.04 1.08l-4.5-4.25a.75.75 0 010-1.08l4.5-4.25a.75.75 0 011.06.02z"></path>
								</svg>
							</div>
						</Hint>
					{/if}
				</div> -->

				<!-- forward chevron -->
				<!-- <div class="z-30 float-left">
					{#if num_back > 0}
						<Hint placement="top" text="Press > to go forward">
							<button
								on:click={undoGetLastCard}
								class="float-right cursor-pointer ring-columbia focus:outline-none focus:ring duration-75">
								<svg class="dark:invert flex-none h-7 w-7 cursor-pointer  "
									fill="currentColor" viewBox="0 0 20 20" xmlns="http://www.w3.org/2000/svg" aria-hidden="true" data-darkreader-inline-fill="">
									<path clip-rule="evenodd" fill-rule="evenodd" d="M7.21 14.77a.75.75 0 01.02-1.06L11.168 10 7.23 6.29a.75.75 0 111.04-1.08l4.5 4.25a.75.75 0 010 1.08l-4.5 4.25a.75.75 0 01-1.06-.02z"></path>
								</svg>
							</button>
						</Hint>
					{:else}
						<Hint placement="top" text="Press > to go forward">
							<div
								class="float-right cursor-default opacity-50">
								<svg class="dark:invert flex-none h-7 w-7 "
									fill="currentColor" viewBox="0 0 20 20" xmlns="http://www.w3.org/2000/svg" aria-hidden="true" data-darkreader-inline-fill="">
									<path clip-rule="evenodd" fill-rule="evenodd" d="M7.21 14.77a.75.75 0 01.02-1.06L11.168 10 7.23 6.29a.75.75 0 111.04-1.08l4.5 4.25a.75.75 0 010 1.08l-4.5 4.25a.75.75 0 01-1.06-.02z"></path>
								</svg>
							</div>
						</Hint>
					{/if}
				</div> -->
				
			</div>

			<!-- question circle -->
			<!-- <div class="z-30 absolute right-2 bottom-0">
				<Hint placement="left" text="Move all cards from the middle stack (new cards) and right stack (reviewed cards) to the left stack by practicing">
					<div class="float-right cursor-default">
						<svg class="dark:invert flex-none h-6 w-6 outline-columbia focus:outline"
							fill="currentColor" viewBox="0 0 20 20" xmlns="http://www.w3.org/2000/svg" aria-hidden="true">
							<path clip-rule="evenodd" fill-rule="evenodd" d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zM8.94 6.94a.75.75 0 11-1.061-1.061 3 3 0 112.871 5.026v.345a.75.75 0 01-1.5 0v-.5c0-.72.57-1.172 1.081-1.287A1.5 1.5 0 108.94 6.94zM10 15a1 1 0 100-2 1 1 0 000 2z"></path>
						</svg>
					</div>
				</Hint>
			</div> -->

			<!-- debugging stack height -->
			<!-- <div class="absolute right-5 bottom-0 dark:invert">
                {windowHeight}
				({state.buf.idx} / {state.buf.data.length})
			</div> -->

		<!-- </div> -->