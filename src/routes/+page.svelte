<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { listen } from '@tauri-apps/api/event';
  import { invoke } from '@tauri-apps/api/core';
  
  import ClipboardList from '../components/ClipboardList.svelte';
  import ContentEditor from '../components/ContentEditor.svelte';

  interface ClipboardEntry {
    id: number;
    content: string;
    timestamp: string;
    content_type: string;
  }

  let clipboardEntries: ClipboardEntry[] = [];
  let selectedEntry: string = '';
  let selectedIndex: number = -1;
  let jsonValidationResult: { 
    is_valid: boolean; 
    error_message?: string; 
    formatted_json?: string; 
    line_number?: number;
    column_number?: number;
    parsedJson?: any;
  } | null = null;
  let autoValidateJson: boolean = false;
  let viewMode: 'formatted' | 'tree' | 'none' = 'none';
  let jsonSearchQuery: string = '';
  let filteredJsonData: any = null;
  let isEditing: boolean = false;
  let editingTimeout: number | null = null;

  onMount(async () => {
      try {
        const history = await invoke('get_clipboard_history_entries', { limit: 20 }) as ClipboardEntry[];

      clipboardEntries = history;
      
      if (clipboardEntries.length > 0 && selectedIndex === -1) {
        selectEntry(0);
      }
    } catch (error) {
      console.error('Failed to load clipboard history:', error);
    }

    // Listen for new clipboard updates
    listen<string>('clipboard-update', (event) => {
      // Skip adding to clipboard list if user is actively editing
      if (isEditing) {
        console.log('Skipping clipboard update during editing:', event.payload);
        return;
      }
      // Create a new entry with temporary ID (backend will handle proper ID)
      const newEntry: ClipboardEntry = {
        id: Date.now(), // Temporary ID
        content: event.payload,
        timestamp: new Date().toISOString(),
        content_type: 'text' // Will be properly classified by backend
      };
      
      clipboardEntries = [newEntry, ...clipboardEntries];
      // Auto-select the first (newest) entry if none is selected
      if (selectedIndex === -1) {
        selectEntry(0);
      }
    });

    // Listen for historical entries from database
    listen<ClipboardEntry>('clipboard-history', (event) => {
      // This handles entries loaded from database on startup
      const existingIndex = clipboardEntries.findIndex(entry => entry.id === event.payload.id);
      if (existingIndex === -1) {
        clipboardEntries = [...clipboardEntries, event.payload];
      }
    });
  });

  // Cleanup editing timeout on component destroy
  onDestroy(() => {
    if (editingTimeout) {
      clearTimeout(editingTimeout);
    }
  });

  async function copyToClipboard(content: string) {
    try {
      await invoke('copy_to_clipboard', { text: content });
    } catch (error) {
      console.error('Failed to copy to clipboard:', error);
    }
  }

  function selectEntry(index: number) {
    selectedIndex = index;
    selectedEntry = clipboardEntries[index]?.content || '';
    // Auto-validate JSON when selecting new entry
    if (autoValidateJson && isLikelyJson(selectedEntry)) {
      validateJson();
    } else {
      jsonValidationResult = null;
    }
  }

  function updateEntry() {
    if (selectedIndex >= 0 && clipboardEntries[selectedIndex]) {
      clipboardEntries[selectedIndex].content = selectedEntry;
      clipboardEntries = [...clipboardEntries]; // Trigger reactivity
      // Re-validate JSON if auto-validation is enabled
      if (autoValidateJson && isLikelyJson(selectedEntry)) {
        validateJson();
      } else if (!isLikelyJson(selectedEntry)) {
        jsonValidationResult = null;
      }
    }
  }

  async function validateJson() {
    try {
      const result = await invoke('validate_json', { jsonText: selectedEntry });
      jsonValidationResult = result as typeof jsonValidationResult;
      
      if (jsonValidationResult?.is_valid && jsonValidationResult.formatted_json) {
        // Parse the formatted JSON to get the actual object for tree view
        try {
          jsonValidationResult.parsedJson = JSON.parse(jsonValidationResult.formatted_json);
        } catch (e) {
          console.error('Failed to parse formatted JSON:', e);
        }
      }
      
      updateFilteredJson();
      return jsonValidationResult;
    } catch (error) {
      console.error('Validation failed:', error);
      jsonValidationResult = {
        is_valid: false,
        error_message: `Validation error: ${error}`,
      };
      filteredJsonData = null;
      return jsonValidationResult;
    }
  }

  function updateFilteredJson() {
    if (!jsonValidationResult?.parsedJson) {
      filteredJsonData = null;
      return;
    }

    if (!jsonSearchQuery.trim()) {
      filteredJsonData = jsonValidationResult.parsedJson;
      return;
    }

    filteredJsonData = filterJsonBySearch(jsonValidationResult.parsedJson, jsonSearchQuery.toLowerCase());
  }

  function filterJsonBySearch(obj: any, query: string): any {
    if (obj === null || obj === undefined) return obj;
    
    if (typeof obj === 'string') {
      return obj.toLowerCase().includes(query) ? obj : null;
    }
    
    if (typeof obj === 'number' || typeof obj === 'boolean') {
      return String(obj).toLowerCase().includes(query) ? obj : null;
    }

    if (Array.isArray(obj)) {
      const filteredArray = obj
        .map((item, index) => filterJsonBySearch(item, query))
        .filter((item, index) => {
          // Keep if the item matches or if the index matches the query
          return item !== null || String(index).includes(query);
        });
      return filteredArray.length > 0 ? filteredArray : null;
    }

    if (typeof obj === 'object') {
      const filteredObj: any = {};
      let hasMatches = false;

      for (const [key, value] of Object.entries(obj)) {
        const keyMatches = key.toLowerCase().includes(query);
        const filteredValue = filterJsonBySearch(value, query);
        
        if (keyMatches || filteredValue !== null) {
          filteredObj[key] = keyMatches ? value : filteredValue;
          hasMatches = true;
        }
      }

      return hasMatches ? filteredObj : null;
    }

    return obj;
  }

  function clearJsonSearch() {
    jsonSearchQuery = '';
    updateFilteredJson();
  }

  function startEditing() {
    isEditing = true;
    
    // Clear any existing timeout
    if (editingTimeout) {
      clearTimeout(editingTimeout);
    }
    
    // Set editing to false after 3 seconds of inactivity
    editingTimeout = setTimeout(() => {
      isEditing = false;
      console.log('Editing mode automatically disabled after inactivity');
    }, 3000);
  }

  function stopEditing() {
    isEditing = false;
    if (editingTimeout) {
      clearTimeout(editingTimeout);
      editingTimeout = null;
    }
  }

  // Reactive statement to update filtered data when search query changes
  $: if (jsonSearchQuery !== undefined) {
    updateFilteredJson();
  }


  async function deleteClipboardEntry(index: number) {
    try {
      const entryToDelete = clipboardEntries[index];
      if (!entryToDelete) return;
      
      const result = await invoke('delete_clipboard_entry', { entryId: entryToDelete.id }) as { success: boolean; message: string };
      
      if (!result.success) {
        console.error('Failed to delete clipboard entry:', result.message);
        return;
      }
      
      console.log('Successfully deleted entry:', result.message);
      
      clipboardEntries = clipboardEntries.filter((_, i) => i !== index);
      
      if (selectedIndex === index) {
        selectedIndex = -1;
        selectedEntry = '';
        jsonValidationResult = null;
        filteredJsonData = null;
      } else if (selectedIndex > index) {
        selectedIndex--;
      }
    } catch (error) {
      console.error('Failed to delete clipboard entry:', error);
    }
  }

  async function clearAllEntries() {
    if (clipboardEntries.length === 0) return;
    
    const confirmed = confirm(`Are you sure you want to delete all ${clipboardEntries.length} clipboard entries? This action cannot be undone.`);
    
    if (confirmed) {
      try {
        const result = await invoke('clear_all_clipboard_entries') as { success: boolean; message: string };
        
        if (!result.success) {
          console.error('Failed to clear clipboard entries:', result.message);
          return;
        }
        
        console.log('Successfully cleared all entries:', result.message);
        
        clipboardEntries = [];
        selectedIndex = -1;
        selectedEntry = '';
        jsonValidationResult = null;
        filteredJsonData = null;
        jsonSearchQuery = '';
        
      } catch (error) {
        console.error('Failed to clear clipboard entries:', error);
      }
    }
  }

  async function formatJson() {
    try {
      const result = await invoke('format_json', { jsonText: selectedEntry });
      selectedEntry = result as string;
      updateEntry();
    } catch (error) {
      console.error('Failed to format JSON:', error);
    }
  }

  async function minifyJson() {
    try {
      const result = await invoke('minify_json', { jsonText: selectedEntry });
      selectedEntry = result as string;
      updateEntry();
    } catch (error) {
      console.error('Failed to minify JSON:', error);
    }
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

  function toggleAutoValidation() {
    autoValidateJson = !autoValidateJson;
    if (autoValidateJson && isLikelyJson(selectedEntry)) {
      validateJson();
    } else if (!autoValidateJson) {
      jsonValidationResult = null;
    }
  }

  function isLikelyJson(text: string): boolean {
    const trimmed = text.trim();
    return (trimmed.startsWith('{') && trimmed.endsWith('}')) || 
           (trimmed.startsWith('[') && trimmed.endsWith(']'));
  }

  async function loadSampleEntries() {
    await invoke('add_sample_entries');
  }


  function handleLoadSampleEntries() {
    loadSampleEntries();
  }

  function handleClearAllEntries() {
    clearAllEntries();
  }

  function handleSelectEntry(event: CustomEvent<{ index: number }>) {
    selectEntry(event.detail.index);
  }

  function handleCopyEntry(event: CustomEvent<{ content: string }>) {
    copyToClipboard(event.detail.content);
  }

  function handleDeleteEntry(event: CustomEvent<{ index: number }>) {
    deleteClipboardEntry(event.detail.index);
  }

  function handleCopyContent(event: CustomEvent<{ content: string }>) {
    copyToClipboard(event.detail.content);
  }

  function handleSave() {
    updateEntry();
    stopEditing(); // Stop editing mode when user saves
  }

  function handleToggleAutoValidation() {
    toggleAutoValidation();
  }

  function handleFormatOrMinify() {
    if (isLikelyJson(selectedEntry) && isJsonMinified(selectedEntry)) {
      formatJson();
    } else {
      minifyJson();
    }
  }

  function handleValidate() {
    validateJson();
  }

  function handleToggleView() {
    viewMode = viewMode === 'formatted' ? 'tree' : 'formatted';
  }

  function handleUpdateEntry(event: CustomEvent<{ content: string }>) {
    selectedEntry = event.detail.content;
    startEditing(); // Track that user is actively editing
    updateEntry();
  }

  function handleClearJsonSearch() {
    clearJsonSearch();
  }

  function handleToggleEditingMode() {
    if (isEditing) {
      stopEditing();
    } else {
      startEditing();
    }
  }
</script>

<main class=" h-screen p-4 font-sans">
  <div class="flex  gap-4 flex-1 h-full">
    <ClipboardList
      entries={clipboardEntries.map(entry => entry.content)}
      {selectedIndex}
      on:selectEntry={handleSelectEntry}
      on:copyEntry={handleCopyEntry}
      on:deleteEntry={handleDeleteEntry}
    />

    <ContentEditor
      {selectedEntry}
      {selectedIndex}
      {autoValidateJson}
      {viewMode}
      {jsonValidationResult}
      {isEditing}
      bind:jsonSearchQuery
      {filteredJsonData}
      on:copy={handleCopyContent}
      on:save={handleSave}
      on:toggleAutoValidation={handleToggleAutoValidation}
      on:formatOrMinify={handleFormatOrMinify}
      on:validate={handleValidate}
      on:toggleView={handleToggleView}
      on:updateEntry={handleUpdateEntry}
      on:clearJsonSearch={handleClearJsonSearch}
      on:toggleEditingMode={handleToggleEditingMode}
    />
  </div>
</main>

<style>
  @import "./index.css";
</style>