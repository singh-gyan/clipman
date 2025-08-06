<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { ScrollArea } from '$lib/components/ui/scroll-area';
  import ClipboardEntry from './ClipboardEntry.svelte';
  import { Button } from '$lib/components/ui/button';
  import { Hamburger, Menu } from '@lucide/svelte';

  const dispatch = createEventDispatcher<{
    selectEntry: { index: number };
    copyEntry: { content: string };
    deleteEntry: { index: number };
  }>();

  export let entries: string[] = [];
  export let selectedIndex: number = -1;
  export let showClipBoardList: boolean = true;

  function handleEntrySelect(event: CustomEvent<{ index: number }>) {
    dispatch('selectEntry', event.detail);
  }

  function handleEntryCopy(event: CustomEvent<{ content: string }>) {
    dispatch('copyEntry', event.detail);
  }

  function handleEntryDelete(event: CustomEvent<{ index: number }>) {
    dispatch('deleteEntry', event.detail);
  }

  function handleTogglePanel() {
    showClipBoardList = !showClipBoardList;
  }
</script>



  <Button class="relative top-2 right-2" variant="ghost" size="icon" onclick={handleTogglePanel}>
    <Menu />
  </Button>
<div class="flex flex-col min-w-0 transition-all duration-300 ease-in-out overflow-hidden {showClipBoardList ? 'flex-1 min-w-[300px]' : 'flex-none max-w-0 min-w-0 !m-0 !p-0 !border-0'} md:max-h-none {showClipBoardList ? 'md:min-w-auto' : 'md:hidden'}">
  <ScrollArea class="flex-1 pr-2 overflow-y-auto">
    <div class="flex flex-col gap-2">
      {#each entries as entry, index}
        <ClipboardEntry
          {entry}
          {index}
          isSelected={selectedIndex === index}
          on:select={handleEntrySelect}
          on:copy={handleEntryCopy}
          on:delete={handleEntryDelete}
        />
      {/each}
      {#if entries.length === 0}
        <div class="flex items-center justify-center h-48 text-muted-foreground text-center bg-muted rounded-lg border-2 border-dashed border-border">
          <p>No clipboard entries yet. Copy something to get started!</p>
        </div>
      {/if}
    </div>
  </ScrollArea>
</div>

<style>
  @import '../styles/ClipboardList.css';
</style> 