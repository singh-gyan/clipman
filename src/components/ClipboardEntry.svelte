<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import { Button } from "$lib/components/ui/button";
  import { Clipboard, ClipboardCheck, Trash } from "@lucide/svelte";
  const dispatch = createEventDispatcher<{
    select: { index: number };
    copy: { content: string };
    delete: { index: number };
  }>();

  export let entry: string;
  export let index: number;
  export let isSelected: boolean = false;
  let isCopied: boolean = false;

  function handleSelect() {
    dispatch("select", { index });
  }

  function handleCopy(event: Event) {
    event.stopPropagation();
    dispatch("copy", { content: entry });
    isCopied = true;
    setTimeout(() => {
      isCopied = false;
    }, 2000);
  }

  function handleDelete(event: Event) {
    event.stopPropagation();
    dispatch("delete", { index });
  }

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === "Enter") {
      handleSelect();
    }
  }
</script>

<div
  class="bg-secondary border-2 border-transparent rounded-lg p-3 flex justify-between items-start cursor-pointer transition-all duration-200 min-h-[60px] hover:bg-blue-50 hover:border-blue-200 focus:outline-none focus:border-primary focus:ring-2 focus:ring-primary/30 {isSelected
    ? 'bg-primary text-primary-foreground border-primary/80'
    : ''}"
  on:click={handleSelect}
  on:keydown={handleKeydown}
  role="button"
  tabindex="0"
  title="Click to select entry"
>
  <div class="flex-1 mr-2">
    <div class="mb-1">
      <!-- Reserved for future features like JSON indicator -->
    </div>
    <pre
      class="text-sm leading-relaxed whitespace-nowrap overflow-hidden text-ellipsis m-0"
      style="max-width: 185px;">{entry}</pre>
  </div>
<div class='relative top-0 right-0'>
  <div class="flex gap-2 items-center flex-shrink-0">
    <Button
      variant="ghost"
      size="sm"
      class="h-8 w-8 p-0 {
         'transition  hover:bg-green-50 hover:text-green-600 '}"
      onclick={handleCopy}
      title="Copy to clipboard"
      aria-label="Copy to clipboard"
    >
      {#if isCopied}
        <ClipboardCheck  color='#13c96b' />
      {:else}
        <Clipboard />
      {/if}
    </Button>
    <Button
      variant="ghost"
      size="sm"
      class="h-8 w-8 p-0 opacity-70 hover:opacity-100 {
        'transition  hover:bg-red-50 hover:text-red-600'}"
      onclick={handleDelete}
      title="Delete this entry"
      aria-label="Delete clipboard entry"
    >
      <Trash />
    </Button>
  </div>
</div>
</div>