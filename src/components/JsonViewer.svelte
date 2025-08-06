<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { Button } from '$lib/components/ui/button';
  import { Input } from '$lib/components/ui/input';
  import TreeView from 'svelte-tree-view';

  const dispatch = createEventDispatcher<{
    clearSearch: void;
  }>();

  export let selectedEntry: string = '';
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

  function isLikelyJson(text: string): boolean {
    const trimmed = text.trim();
    return (trimmed.startsWith('{') && trimmed.endsWith('}')) || 
           (trimmed.startsWith('[') && trimmed.endsWith(']'));
  }

  function clearJsonSearch() {
    jsonSearchQuery = '';
    dispatch('clearSearch');
  }

  $: isJson = isLikelyJson(selectedEntry);
</script>

{#if isJson }
  <div class="flex-1 flex flex-col min-w-0 bg-muted/50 border border-border rounded-lg p-4">
    <h3 class="m-0 mb-3 text-gray-600 text-base border-b border-gray-200 pb-2">JSON Viewer</h3>
    {#if jsonValidationResult}
      <div class="p-3 mb-2 rounded border font-mono text-sm leading-relaxed overflow-x-auto {jsonValidationResult.is_valid ? 'bg-green-50 border-green-200 text-green-800' : 'bg-red-50 border-red-200 text-red-800'}">
        {#if jsonValidationResult.is_valid}
          <p class="m-0 mb-2">✅ JSON is valid!</p>
          
          <!-- JSON Search Input -->
          {#if viewMode === 'tree'}
            <div class="relative mb-3">
              <Input 
                type="text" 
                bind:value={jsonSearchQuery}
                placeholder="🔍 Search in JSON keys/values..."
                class="w-full pr-8 text-sm bg-white transition-colors focus:border-primary focus:ring-2 focus:ring-primary/30"
              />
              {#if jsonSearchQuery}
                <button 
                  class="absolute right-2 top-1/2 transform -translate-y-1/2 bg-none border-none text-muted-foreground cursor-pointer text-base p-1 rounded hover:bg-gray-100 hover:text-gray-700 transition-colors"
                  onclick={clearJsonSearch}
                  title="Clear search"
                >
                  ✕
                </button>
              {/if}
            </div>
          {/if}

          {#if viewMode === 'formatted'}
            <pre class="mt-2 p-3 bg-gray-900 text-gray-300 border border-gray-700 rounded text-xs leading-relaxed overflow-x-auto whitespace-pre font-mono shadow-inner">{jsonValidationResult.formatted_json}</pre>
          {:else}
            <div class="flex-1 overflow-y-auto bg-gray-900 rounded-lg p-4 border border-gray-700" style="
              --tree-view-base00: #1e1e1e;
              --tree-view-base01: #383838;
              --tree-view-base02: #404040;
              --tree-view-base03: #808080;
              --tree-view-base04: #b4b7b4;
              --tree-view-base05: #d4d4d4;
              --tree-view-base06: #e0e0e0;
              --tree-view-base07: #ffffff;
              --tree-view-base08: #ff6b6b;
              --tree-view-base09: #ffaa44;
              --tree-view-base0A: #ffd700;
              --tree-view-base0B: #ce9178;
              --tree-view-base0C: #4ec9b0;
              --tree-view-base0D: #9cdcfe;
              --tree-view-base0E: #569cd6;
              --tree-view-base0F: #d19a66;
              color: #d4d4d4;
              font-family: Monaco, Menlo, 'Courier New', monospace;
              font-size: 14px;
            ">
              {#if filteredJsonData !== null && filteredJsonData !== undefined}
                <TreeView data={filteredJsonData} />
              {:else if jsonSearchQuery}
                <div class="flex flex-col items-center justify-center h-48 text-gray-400 text-center gap-4">
                  <p class="m-0 italic">🔍 No matches found for "{jsonSearchQuery}"</p>
                  <Button variant="outline" size="sm" onclick={clearJsonSearch}>Clear Search</Button>
                </div>
              {:else}
                <TreeView data={jsonValidationResult.parsedJson} />
              {/if}
            </div>
          {/if}
        {:else}
          <div class="bg-red-900/20 border border-red-600 rounded p-4 mt-2">
            <p class="m-0 mb-2 text-red-400">❌ JSON is invalid: {jsonValidationResult.error_message}</p>
            {#if jsonValidationResult.line_number}
              <p class="m-0 font-mono text-sm text-orange-300 bg-orange-300/10 px-2 py-1 rounded inline-block">📍 Line {jsonValidationResult.line_number}{jsonValidationResult.column_number ? `, Column ${jsonValidationResult.column_number}` : ''}</p>
            {/if}
          </div>
        {/if}
      </div>
    {:else}
      <div class="flex items-center justify-center h-full text-muted-foreground text-center italic">
        <p>Click "Validate" to check JSON syntax</p>
      </div>
    {/if}
  </div>
{:else}
  {#if viewMode !== 'none'}
  <div class="flex-1 flex flex-col min-w-0 bg-muted/30 border border-border rounded-lg p-4 opacity-60">
    <h3 class="m-0 mb-3 text-gray-600 text-base border-b border-gray-200 pb-2">JSON Viewer</h3>
    <div class="flex items-center justify-center h-full text-muted-foreground text-center italic">
        <p>Select JSON content to view formatted output</p>
      </div>
    </div>
  {/if}
{/if} 