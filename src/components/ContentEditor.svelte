<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { Textarea } from '$lib/components/ui/textarea';
  import Toolbar from './Toolbar.svelte';
  import JsonViewer from './JsonViewer.svelte';

  const dispatch = createEventDispatcher<{
    copy: { content: string };
    save: void;
    toggleAutoValidation: void;
    formatOrMinify: void;
    validate: void;
    toggleView: void;
    updateEntry: { content: string };
    clearJsonSearch: void;
    toggleEditingMode: void;
  }>();

  export let selectedEntry: string = '';
  export let selectedIndex: number = -1;
  export let viewMode: 'formatted' | 'tree' | 'none' = 'none';
  export let jsonValidationResult: { 
    is_valid: boolean; 
    error_message?: string; 
    formatted_json?: string; 
    line_number?: number;
    column_number?: number;
    parsedJson?: any;
  } | null = null;
  export let jsonSearchQuery: string = '';
  export let filteredJsonData: any = null;
  export let isEditing: boolean = false;

  function handleCopy() {
    dispatch('copy', { content: selectedEntry });
  }

  function handleSave() {
    dispatch('save');
  }

  function handleToggleAutoValidation() {
    dispatch('toggleAutoValidation');
  }

  function handleFormatOrMinify() {
    dispatch('formatOrMinify');
  }

  function handleValidate() {
    dispatch('validate');
  }

  function handleToggleView() {
    dispatch('toggleView');
  }

  function handleInput(event: Event) {
    const target = event.target as HTMLTextAreaElement;
    selectedEntry = target.value;
    dispatch('updateEntry', { content: selectedEntry });
  }

  function handleClearJsonSearch() {
    dispatch('clearJsonSearch');
  }

  function handleToggleEditingMode() {
    dispatch('toggleEditingMode');
  }
</script>

<div class="flex-[2] flex flex-col min-w-[400px] md:min-w-0">
  <div class="flex items-center justify-between mb-2">
    <h2 class="text-muted-foreground text-xl font-medium">Selected Content</h2>
    {#if isEditing}
      <div class="flex items-center gap-2 text-sm text-orange-600 bg-orange-50 px-2 py-1 rounded-md border border-orange-200">
        <div class="w-2 h-2 bg-orange-500 rounded-full animate-pulse"></div>
        Editing Mode - Clipboard updates paused
      </div>
    {/if}
  </div>
  {#if selectedIndex >= 0}
    <div class="flex flex-col h-full">
      <Toolbar
        {selectedEntry}
        {viewMode}
        on:copy={handleCopy}
        on:save={handleSave}
        on:toggleAutoValidation={handleToggleAutoValidation}
        on:formatOrMinify={handleFormatOrMinify}
        on:validate={handleValidate}
        on:toggleView={handleToggleView}
        on:toggleEditingMode={handleToggleEditingMode}
      />
      
      <!-- Split view: Editor + JSON Viewer -->
      <div class="flex gap-4 flex-1 min-h-0 lg:flex-row flex-col">
        <!-- Left: Text Editor -->
        <div class="flex-1 flex flex-col min-w-0">
          <Textarea 
            bind:value={selectedEntry}
            class="flex-1 w-full min-h-[300px] p-4 border-2 border-border rounded-lg font-mono text-sm leading-relaxed resize-none outline-none transition-colors focus:border-primary focus:ring-2 focus:ring-primary/30"
            placeholder="Select a clipboard entry to edit..."
            oninput={handleInput}
          />
        </div>
        
        <JsonViewer
          {selectedEntry}
          {viewMode}
          {jsonValidationResult}
          bind:jsonSearchQuery
          {filteredJsonData}
          on:clearSearch={handleClearJsonSearch}
        />
      </div>
    </div>
  {:else}
    <div class="flex items-center justify-center h-full text-muted-foreground text-center bg-muted rounded-lg border-2 border-dashed border-border">
      <p>Select a clipboard entry from the left panel to view and edit it here.</p>
    </div>
  {/if}
</div> 