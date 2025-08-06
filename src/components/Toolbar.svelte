<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { Button } from '$lib/components/ui/button';
  import { FileJson, Save } from '@lucide/svelte';

  const dispatch = createEventDispatcher<{
    copy: void;
    save: void;
    toggleAutoValidation: void;
    formatOrMinify: void;
    validate: void;
    toggleView: void;
    toggleEditingMode: void;
  }>();

  export let selectedEntry: string = '';
  // export let autoValidateJson: boolean = true;
  export let viewMode: 'formatted' | 'tree' | 'none' = 'none';
  // export let isEditing: boolean = false;

  function isLikelyJson(text: string): boolean {
    const trimmed = text.trim();
    return (trimmed.startsWith('{') && trimmed.endsWith('}')) || 
           (trimmed.startsWith('[') && trimmed.endsWith(']'));
  }

  function isJsonMinified(jsonString: string): boolean {
    try {
      const parsed = JSON.parse(jsonString);
      const formatted = JSON.stringify(parsed, null, 2);
      return jsonString.length < formatted.length * 0.8; // Rough heuristic
    } catch {
      return false;
    }
  }

  $: isJson = isLikelyJson(selectedEntry);
  $: isMinified = isJson && isJsonMinified(selectedEntry);
</script>

<div class="flex gap-2 mb-2 p-2 bg-secondary rounded-md flex-wrap">
  <Button 
    variant="default"
    size="sm"
    onclick={() => dispatch('save')}
    title="Save changes"
    class="text-xs whitespace-nowrap inline-flex items-center gap-1  hover:bg-blue-100  hover:color-white"
  >
  <Save />
  </Button>
  
  <!-- <Button 
    variant="secondary"
    size="sm"
    onclick={() => dispatch('toggleAutoValidation')}
    title={autoValidateJson ? 'Disable JSON auto-validation' : 'Enable JSON auto-validation'}
    class="text-xs whitespace-nowrap inline-flex items-center gap-1 bg-purple-600 hover:bg-purple-700 text-white"
  >
    {autoValidateJson ? '🔍' : '👁️'} Auto-Validate
  </Button>
   -->

  {#if isJson}
    <Button 
      variant="secondary"
      size="sm"
      onclick={() => dispatch('formatOrMinify')}
      title={isMinified ? "Format JSON (pretty-print)" : "Minify JSON (compact)"}
      class="text-xs whitespace-nowrap inline-flex items-center gap-1 bg-green-600 hover:bg-green-700 text-white"
    >
      {isMinified ? '🧹 Format' : '📦 Minify'}
    </Button>
    
    <Button 
      variant="secondary"
      size="sm"
      onclick={() => dispatch('validate')}
      title="Validate JSON"
      class="text-xs whitespace-nowrap inline-flex items-center gap-1 bg-green-600 hover:bg-green-700 text-white"
    >
<FileJson />
    </Button>
    
    <Button 
      variant="secondary"
      size="sm"
      onclick={() => dispatch('toggleView')}
      title={viewMode === 'formatted' ? 'Switch to tree view' : 'Switch to formatted view'}
      class="text-xs whitespace-nowrap inline-flex items-center gap-1 bg-slate-600 hover:bg-slate-700 text-white"
    >
      {viewMode === 'formatted' ? '🌳' : '📄'} {viewMode === 'formatted' ? 'Tree' : 'Format'}
    </Button>
  {/if}
</div> 