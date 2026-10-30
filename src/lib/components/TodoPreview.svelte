<script lang="ts">
  import IslandHeader from './IslandHeader.svelte';
  import { islandState } from '../state/island.svelte';
  import type { TodoItem, TodoPriority } from '../types';

  let newTaskTitle = $state('');
  let newTaskPriority = $state<TodoPriority>('medium');
  let newTaskDue = $state('');
  let isDatePickerOpen = $state(false);
  let activeFilter = $state<'all' | 'pending' | 'completed'>('all');
  let searchQuery = $state('');
  let feedbackMessage = $state<{ type: 'success' | 'error'; text: string } | null>(null);
  let feedbackTimer: ReturnType<typeof setTimeout> | null = null;
  let isExporting = $state(false);
  let isSyncing = $state(false);
  let datePickerAnchorEl: HTMLElement | null = $state(null);

  function handleWindowClick(e: MouseEvent) {
    if (isDatePickerOpen && datePickerAnchorEl && !datePickerAnchorEl.contains(e.target as Node)) {
      isDatePickerOpen = false;
    }
  }

  // Calendar state for custom picker
  const today = new Date();
  let calYear = $state(today.getFullYear());
  let calMonth = $state(today.getMonth()); // 0-indexed

  const MONTH_NAMES = [
    'January', 'February', 'March', 'April', 'May', 'June',
    'July', 'August', 'September', 'October', 'November', 'December'
  ];

  const todos = $derived(islandState.todos);
  const pendingCount = $derived(islandState.pendingTodosCount);
  const totalCount = $derived(islandState.totalTodosCount);
  const completedCount = $derived(totalCount - pendingCount);

  function formatDateStr(d: Date): string {
    const y = d.getFullYear();
    const m = String(d.getMonth() + 1).padStart(2, '0');
    const day = String(d.getDate()).padStart(2, '0');
    return `${y}-${m}-${day}`;
  }

  function getDaysInMonth(year: number, month: number): number {
    return new Date(year, month + 1, 0).getDate();
  }

  function getFirstDayOfWeek(year: number, month: number): number {
    return new Date(year, month, 1).getDay();
  }

  function selectDate(dateStr: string) {
    newTaskDue = dateStr;
    isDatePickerOpen = false;
  }

  function setQuickDate(offsetDays: number) {
    const d = new Date();
    d.setDate(d.getDate() + offsetDays);
    selectDate(formatDateStr(d));
  }

  function clearDate(e?: MouseEvent) {
    if (e) e.stopPropagation();
    newTaskDue = '';
  }

  function prevMonth() {
    if (calMonth === 0) {
      calMonth = 11;
      calYear -= 1;
    } else {
      calMonth -= 1;
    }
  }

  function nextMonth() {
    if (calMonth === 11) {
      calMonth = 0;
      calYear += 1;
    } else {
      calMonth += 1;
    }
  }

  function formatDisplayDueDate(dateStr?: string | null): { label: string; isPast: boolean; isToday: boolean } {
    if (!dateStr) return { label: '', isPast: false, isToday: false };

    const todayStr = formatDateStr(new Date());
    const tomorrow = new Date();
    tomorrow.setDate(tomorrow.getDate() + 1);
    const tomorrowStr = formatDateStr(tomorrow);

    if (dateStr === todayStr) {
      return { label: 'Today', isPast: false, isToday: true };
    }
    if (dateStr === tomorrowStr) {
      return { label: 'Tomorrow', isPast: false, isToday: false };
    }

    // Check if YYYY-MM-DD
    const match = dateStr.match(/^(\d{4})-(\d{2})-(\d{2})$/);
    if (match) {
      const parts = dateStr.split('-');
      const d = new Date(parseInt(parts[0]), parseInt(parts[1]) - 1, parseInt(parts[2]));
      const isPast = dateStr < todayStr;
      const formatted = d.toLocaleDateString(undefined, { month: 'short', day: 'numeric' });
      return { label: formatted, isPast, isToday: false };
    }

    return { label: dateStr, isPast: false, isToday: false };
  }

  const filteredTodos = $derived(
    todos.filter((t) => {
      if (activeFilter === 'pending' && t.completed) return false;
      if (activeFilter === 'completed' && !t.completed) return false;
      if (searchQuery.trim()) {
        const q = searchQuery.toLowerCase().trim();
        return t.title.toLowerCase().includes(q) || (t.due_date && t.due_date.toLowerCase().includes(q));
      }
      return true;
    })
  );

  function showFeedback(type: 'success' | 'error', text: string) {
    feedbackMessage = { type, text };
    if (feedbackTimer) clearTimeout(feedbackTimer);
    feedbackTimer = setTimeout(() => {
      feedbackMessage = null;
    }, 3500);
  }

  async function handleAddTodo(e?: Event) {
    if (e) e.preventDefault();
    const clean = newTaskTitle.trim();
    if (!clean) return;

    try {
      await islandState.addTodo(clean, newTaskPriority, newTaskDue.trim() || undefined);
      newTaskTitle = '';
      newTaskDue = '';
      showFeedback('success', `Task added`);
    } catch (err) {
      showFeedback('error', 'Failed to add task');
    }
  }

  async function handleToggle(id: string) {
    try {
      await islandState.toggleTodo(id);
    } catch (err) {
      showFeedback('error', 'Failed to toggle task');
    }
  }

  async function handleDelete(id: string) {
    try {
      await islandState.deleteTodo(id);
      showFeedback('success', 'Task deleted');
    } catch (err) {
      showFeedback('error', 'Failed to delete task');
    }
  }

  async function handleClearCompleted() {
    try {
      const removed = await islandState.clearCompletedTodos();
      showFeedback('success', `Cleared ${removed} completed task${removed === 1 ? '' : 's'}`);
    } catch (err) {
      showFeedback('error', 'Failed to clear completed tasks');
    }
  }

  async function handleExportMarkdown() {
    isExporting = true;
    try {
      const path = await islandState.exportTodosToMarkdown();
      showFeedback('success', `Exported to ${path}`);
    } catch (err: unknown) {
      const msg = typeof err === 'string' ? err : 'Export failed';
      showFeedback('error', msg);
    } finally {
      isExporting = false;
    }
  }

  async function handleSyncMarkdown() {
    isSyncing = true;
    try {
      const items = await islandState.syncTodosFromMarkdown();
      showFeedback('success', `Synced ${items.length} task${items.length === 1 ? '' : 's'} from ~/todo.md`);
    } catch (err: unknown) {
      const msg = typeof err === 'string' ? err : 'Sync failed';
      showFeedback('error', msg);
    } finally {
      isSyncing = false;
    }
  }

  function getPriorityColor(priority: TodoPriority): string {
    switch (priority) {
      case 'high': return '#ef4444';
      case 'low': return '#10b981';
      default: return '#f59e0b';
    }
  }
</script>

<svelte:window onclick={handleWindowClick} />

<div class="todo-panel">
  <IslandHeader title="To-Do List">
    <span class="header-subtitle-tag">{pendingCount} pending • {completedCount} finished</span>
  </IslandHeader>

  <!-- Action Bar & Markdown Sync -->
  <div class="todo-top-toolbar">
    <div class="filter-tabs">
      <button
        type="button"
        class="tab-btn"
        class:active={activeFilter === 'all'}
        onclick={() => (activeFilter = 'all')}
      >
        All <span class="tab-count">{totalCount}</span>
      </button>
      <button
        type="button"
        class="tab-btn"
        class:active={activeFilter === 'pending'}
        onclick={() => (activeFilter = 'pending')}
      >
        Pending <span class="tab-count">{pendingCount}</span>
      </button>
      <button
        type="button"
        class="tab-btn"
        class:active={activeFilter === 'completed'}
        onclick={() => (activeFilter = 'completed')}
      >
        Done <span class="tab-count">{completedCount}</span>
      </button>
    </div>

    <div class="md-sync-actions">
      <button
        type="button"
        class="tool-btn"
        disabled={isExporting}
        onclick={handleExportMarkdown}
        title="Export tasks to ~/todo.md"
      >
        <span class="icon">📤</span>
        <span>Export MD</span>
      </button>
      <button
        type="button"
        class="tool-btn"
        disabled={isSyncing}
        onclick={handleSyncMarkdown}
        title="Sync tasks from ~/todo.md"
      >
        <span class="icon">📥</span>
        <span>Sync MD</span>
      </button>
      {#if completedCount > 0}
        <button
          type="button"
          class="tool-btn danger"
          onclick={handleClearCompleted}
          title="Remove all finished tasks"
        >
          <span>Clear Done</span>
        </button>
      {/if}
    </div>
  </div>

  <!-- Search & Add Bar -->
  <form class="todo-input-row" onsubmit={handleAddTodo}>
    <input
      type="text"
      class="task-input"
      placeholder="Add a new task... (Press Enter to save)"
      bind:value={newTaskTitle}
    />

    <div class="priority-select-wrapper">
      <select class="priority-select" bind:value={newTaskPriority}>
        <option value="high">🔴 High</option>
        <option value="medium">🟡 Med</option>
        <option value="low">🟢 Low</option>
      </select>
    </div>

    <!-- Date Picker Trigger & Popup -->
    <div class="date-picker-anchor" bind:this={datePickerAnchorEl}>
      <div
        class="due-trigger-wrapper"
        class:has-date={Boolean(newTaskDue)}
        class:active={isDatePickerOpen}
      >
        <button
          type="button"
          class="due-date-trigger-btn"
          onclick={() => (isDatePickerOpen = !isDatePickerOpen)}
          title="Set due date"
        >
          <span class="cal-icon">📅</span>
          <span class="cal-label">
            {newTaskDue ? formatDisplayDueDate(newTaskDue).label : 'Due date'}
          </span>
        </button>
        {#if newTaskDue}
          <button
            type="button"
            class="clear-date-btn"
            onclick={clearDate}
            title="Clear due date"
            aria-label="Clear due date"
          >
            ✕
          </button>
        {/if}
      </div>

      {#if isDatePickerOpen}
        <!-- svelte-ignore a11y_no_static_element_interactions -->
        <div class="datepicker-popover" onkeydown={(e) => { if (e.key === 'Escape') isDatePickerOpen = false; }}>
          <!-- Quick presets -->
          <div class="datepicker-presets">
            <button type="button" class="preset-chip" onclick={() => setQuickDate(0)}>Today</button>
            <button type="button" class="preset-chip" onclick={() => setQuickDate(1)}>Tomorrow</button>
            <button type="button" class="preset-chip" onclick={() => setQuickDate(7)}>+1 Week</button>
            {#if newTaskDue}
              <button type="button" class="preset-chip clear" onclick={() => selectDate('')}>None</button>
            {/if}
          </div>

          <!-- Calendar Header -->
          <div class="calendar-month-bar">
            <button type="button" class="month-nav-btn" onclick={prevMonth} aria-label="Previous month">‹</button>
            <span class="month-label">{MONTH_NAMES[calMonth]} {calYear}</span>
            <button type="button" class="month-nav-btn" onclick={nextMonth} aria-label="Next month">›</button>
          </div>

          <!-- Weekday Labels -->
          <div class="calendar-weekdays">
            <span>Su</span><span>Mo</span><span>Tu</span><span>We</span><span>Th</span><span>Fr</span><span>Sa</span>
          </div>

          <!-- Days Grid -->
          <div class="calendar-days-grid">
            {#each Array(getFirstDayOfWeek(calYear, calMonth)) as _}
              <span class="cal-empty-slot"></span>
            {/each}
            {#each Array(getDaysInMonth(calYear, calMonth)) as _, idx}
              {@const dayNum = idx + 1}
              {@const thisDateStr = `${calYear}-${String(calMonth + 1).padStart(2, '0')}-${String(dayNum).padStart(2, '0')}`}
              {@const isSelected = newTaskDue === thisDateStr}
              {@const isTodayDate = formatDateStr(new Date()) === thisDateStr}
              <button
                type="button"
                class="cal-day-btn"
                class:selected={isSelected}
                class:today={isTodayDate}
                onclick={() => selectDate(thisDateStr)}
              >
                {dayNum}
              </button>
            {/each}
          </div>
        </div>
      {/if}
    </div>

    <button type="submit" class="add-btn" disabled={!newTaskTitle.trim()}>
      <span>+ Add</span>
    </button>
  </form>

  {#if feedbackMessage}
    <div class="feedback-banner {feedbackMessage.type}">
      <span>{feedbackMessage.type === 'success' ? '✓' : '⚠️'} {feedbackMessage.text}</span>
    </div>
  {/if}

  <!-- Task List -->
  <div class="todo-list-scroll">
    {#if filteredTodos.length === 0}
      <div class="empty-todo-state">
        <span class="empty-icon">📋</span>
        <span class="empty-title">
          {#if activeFilter === 'completed'}
            No completed tasks yet. Keep moving forward!
          {:else if activeFilter === 'pending'}
            All tasks completed! Relax or add a new goal above.
          {:else}
            Your to-do list is empty. Add a task to stay focused!
          {/if}
        </span>
        <span class="empty-hint">Use <code>island-cli todo add "Task"</code> to add from terminal</span>
      </div>
    {:else}
      <div class="task-grid">
        {#each filteredTodos as task (task.id)}
          <div class="task-item" class:completed={task.completed}>
            <button
              type="button"
              class="checkbox-btn"
              class:checked={task.completed}
              onclick={() => handleToggle(task.id)}
              aria-label="Toggle task completion"
            >
              {#if task.completed}
                <span class="check-mark">✓</span>
              {/if}
            </button>

            <div class="task-body">
              <span class="task-title" class:strike={task.completed}>{task.title}</span>
              <div class="task-meta">
                <span
                  class="priority-chip"
                  style="--prio-color: {getPriorityColor(task.priority)}"
                >
                  {task.priority.toUpperCase()}
                </span>
                {#if task.due_date}
                  {@const dueInfo = formatDisplayDueDate(task.due_date)}
                  <span
                    class="due-chip"
                    class:is-past={dueInfo.isPast && !task.completed}
                    class:is-today={dueInfo.isToday && !task.completed}
                    title={task.due_date}
                  >
                    <span class="due-icon">{dueInfo.isPast && !task.completed ? '⚠️' : '📅'}</span>
                    <span>{dueInfo.label}</span>
                  </span>
                {/if}
              </div>
            </div>

            <div class="task-actions">
              <button
                type="button"
                class="del-btn"
                onclick={() => handleDelete(task.id)}
                title="Delete task"
              >
                ✕
              </button>
            </div>
          </div>
        {/each}
      </div>
    {/if}
  </div>
</div>

<style>
  .todo-panel {
    display: flex;
    flex-direction: column;
    height: 100%;
    width: 100%;
    gap: 10px;
    box-sizing: border-box;
  }

  .header-subtitle-tag {
    font-size: 10.5px;
    color: var(--island-text-muted, #94a3b8);
    background: rgba(255, 255, 255, 0.08);
    padding: 2px 7px;
    border-radius: 999px;
  }

  .todo-top-toolbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    flex-wrap: wrap;
  }

  .filter-tabs {
    display: flex;
    align-items: center;
    gap: 4px;
    background: rgba(0, 0, 0, 0.25);
    padding: 3px;
    border-radius: 8px;
    border: 1px solid rgba(255, 255, 255, 0.08);
  }

  .tab-btn {
    background: transparent;
    border: none;
    color: var(--island-text-muted, #94a3b8);
    font-size: 11px;
    font-weight: 500;
    padding: 4px 10px;
    border-radius: 6px;
    cursor: pointer;
    display: flex;
    align-items: center;
    gap: 5px;
    transition: all 0.15s ease;
  }

  .tab-btn.active {
    background: rgba(255, 255, 255, 0.12);
    color: #ffffff;
    font-weight: 600;
  }

  .tab-count {
    font-size: 9.5px;
    background: rgba(255, 255, 255, 0.1);
    padding: 1px 5px;
    border-radius: 999px;
  }

  .md-sync-actions {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .tool-btn {
    display: flex;
    align-items: center;
    gap: 4px;
    background: rgba(255, 255, 255, 0.06);
    border: 1px solid rgba(255, 255, 255, 0.1);
    color: var(--island-text-secondary, #cbd5e1);
    font-size: 10.5px;
    padding: 4px 8px;
    border-radius: 6px;
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .tool-btn:hover:not(:disabled) {
    background: rgba(255, 255, 255, 0.14);
    color: #ffffff;
  }

  .tool-btn.danger:hover {
    background: rgba(239, 68, 68, 0.2);
    color: #f87171;
    border-color: rgba(239, 68, 68, 0.4);
  }

  .tool-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .todo-input-row {
    display: flex;
    align-items: center;
    gap: 8px;
    background: rgba(0, 0, 0, 0.28);
    border: 1px solid rgba(255, 255, 255, 0.1);
    padding: 6px 8px;
    border-radius: 9px;
  }

  .task-input {
    flex: 1;
    background: transparent;
    border: none;
    outline: none;
    color: #ffffff;
    font-size: 12px;
    min-width: 140px;
  }

  .task-input::placeholder {
    color: rgba(255, 255, 255, 0.35);
  }

  .priority-select-wrapper {
    position: relative;
  }

  .priority-select {
    background: rgba(255, 255, 255, 0.08);
    border: 1px solid rgba(255, 255, 255, 0.12);
    color: #ffffff;
    font-size: 10.5px;
    padding: 4px 6px;
    border-radius: 6px;
    outline: none;
    cursor: pointer;
  }

  .priority-select option {
    background: #18181b;
    color: #ffffff;
  }

  .date-picker-anchor {
    position: relative;
    display: flex;
    align-items: center;
  }

  .due-trigger-wrapper {
    display: flex;
    align-items: center;
    background: rgba(255, 255, 255, 0.06);
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 6px;
    transition: all 0.15s ease;
    overflow: hidden;
  }

  .due-trigger-wrapper:hover {
    background: rgba(255, 255, 255, 0.1);
    border-color: rgba(255, 255, 255, 0.2);
  }

  .due-trigger-wrapper.has-date {
    color: #38bdf8;
    background: rgba(56, 189, 248, 0.1);
    border-color: rgba(56, 189, 248, 0.25);
  }

  .due-trigger-wrapper.active {
    border-color: var(--island-accent, #10b981);
    background: rgba(255, 255, 255, 0.12);
  }

  .due-date-trigger-btn {
    display: flex;
    align-items: center;
    gap: 5px;
    background: transparent;
    border: none;
    color: inherit;
    font-size: 11px;
    padding: 4px 8px;
    cursor: pointer;
    user-select: none;
    max-width: 130px;
  }

  .due-trigger-wrapper:not(.has-date) .due-date-trigger-btn {
    color: var(--island-text-muted, #94a3b8);
  }

  .due-trigger-wrapper:not(.has-date):hover .due-date-trigger-btn {
    color: #ffffff;
  }

  .cal-icon {
    font-size: 11px;
    opacity: 0.85;
  }

  .cal-label {
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    font-weight: 500;
  }

  .clear-date-btn {
    background: transparent;
    border: none;
    color: inherit;
    font-size: 10px;
    cursor: pointer;
    padding: 0 6px 0 2px;
    margin-right: 3px;
    opacity: 0.7;
    transition: opacity 0.15s, color 0.15s;
    line-height: 1;
  }

  .clear-date-btn:hover {
    opacity: 1;
    color: #ef4444;
  }

  .datepicker-popover {
    position: absolute;
    top: calc(100% + 6px);
    right: 0;
    z-index: 100;
    width: 230px;
    background: rgba(18, 20, 26, 0.95);
    backdrop-filter: blur(16px);
    -webkit-backdrop-filter: blur(16px);
    border: 1px solid rgba(255, 255, 255, 0.15);
    border-radius: 10px;
    padding: 10px;
    box-shadow: 0 12px 30px rgba(0, 0, 0, 0.6);
    display: flex;
    flex-direction: column;
    gap: 8px;
    animation: fadeIn 0.15s ease-out;
  }

  .datepicker-presets {
    display: flex;
    gap: 4px;
    flex-wrap: wrap;
    border-bottom: 1px solid rgba(255, 255, 255, 0.08);
    padding-bottom: 8px;
  }

  .preset-chip {
    background: rgba(255, 255, 255, 0.06);
    border: 1px solid rgba(255, 255, 255, 0.1);
    color: #cbd5e1;
    font-size: 10px;
    padding: 3px 7px;
    border-radius: 5px;
    cursor: pointer;
    transition: all 0.15s;
  }

  .preset-chip:hover {
    background: rgba(255, 255, 255, 0.15);
    color: #ffffff;
  }

  .preset-chip.clear {
    color: #f87171;
    border-color: rgba(239, 68, 68, 0.3);
    background: rgba(239, 68, 68, 0.08);
  }

  .preset-chip.clear:hover {
    background: rgba(239, 68, 68, 0.2);
  }

  .calendar-month-bar {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .month-nav-btn {
    background: transparent;
    border: none;
    color: #94a3b8;
    font-size: 15px;
    font-weight: 700;
    width: 24px;
    height: 24px;
    border-radius: 4px;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all 0.15s;
  }

  .month-nav-btn:hover {
    background: rgba(255, 255, 255, 0.1);
    color: #ffffff;
  }

  .month-label {
    font-size: 11.5px;
    font-weight: 600;
    color: #ffffff;
  }

  .calendar-weekdays {
    display: grid;
    grid-template-columns: repeat(7, 1fr);
    text-align: center;
    font-size: 9.5px;
    font-weight: 600;
    color: #64748b;
    margin-top: 2px;
  }

  .calendar-days-grid {
    display: grid;
    grid-template-columns: repeat(7, 1fr);
    gap: 2px;
  }

  .cal-empty-slot {
    height: 24px;
  }

  .cal-day-btn {
    height: 24px;
    width: 100%;
    border: none;
    background: transparent;
    color: #e2e8f0;
    font-size: 10.5px;
    border-radius: 4px;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all 0.12s ease;
  }

  .cal-day-btn:hover {
    background: rgba(255, 255, 255, 0.12);
    color: #ffffff;
  }

  .cal-day-btn.today {
    border: 1px solid rgba(16, 185, 129, 0.5);
    font-weight: 600;
    color: #34d399;
  }

  .cal-day-btn.selected {
    background: var(--island-accent, #10b981) !important;
    color: #ffffff !important;
    font-weight: 700;
    box-shadow: 0 2px 6px rgba(16, 185, 129, 0.4);
  }

  .add-btn {
    background: var(--island-accent, #10b981);
    border: none;
    color: #ffffff;
    font-size: 11px;
    font-weight: 600;
    padding: 5px 12px;
    border-radius: 6px;
    cursor: pointer;
    transition: opacity 0.15s ease;
    white-space: nowrap;
  }

  .add-btn:hover:not(:disabled) {
    opacity: 0.9;
  }

  .add-btn:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  .feedback-banner {
    display: flex;
    align-items: center;
    font-size: 11px;
    padding: 4px 10px;
    border-radius: 6px;
    animation: fadeIn 0.15s ease-out;
  }

  .feedback-banner.success {
    background: rgba(16, 185, 129, 0.15);
    color: #34d399;
    border: 1px solid rgba(16, 185, 129, 0.3);
  }

  .feedback-banner.error {
    background: rgba(239, 68, 68, 0.15);
    color: #f87171;
    border: 1px solid rgba(239, 68, 68, 0.3);
  }

  .todo-list-scroll {
    flex: 1;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: 6px;
    padding-right: 4px;
    min-height: 0;
  }

  .task-grid {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .task-item {
    display: flex;
    align-items: center;
    gap: 10px;
    background: rgba(255, 255, 255, 0.04);
    border: 1px solid rgba(255, 255, 255, 0.08);
    padding: 8px 12px;
    border-radius: 8px;
    transition: background 0.15s ease, border-color 0.15s ease;
  }

  .task-item:hover {
    background: rgba(255, 255, 255, 0.07);
    border-color: rgba(255, 255, 255, 0.15);
  }

  .task-item.completed {
    opacity: 0.65;
    background: rgba(0, 0, 0, 0.2);
  }

  .checkbox-btn {
    width: 20px;
    height: 20px;
    border-radius: 6px;
    border: 1.5px solid rgba(255, 255, 255, 0.3);
    background: transparent;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
    transition: all 0.15s ease;
    padding: 0;
  }

  .checkbox-btn:hover {
    border-color: var(--island-accent, #10b981);
  }

  .checkbox-btn.checked {
    background: var(--island-accent, #10b981);
    border-color: var(--island-accent, #10b981);
    color: #ffffff;
  }

  .check-mark {
    font-size: 12px;
    font-weight: 700;
  }

  .task-body {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
  }

  .task-title {
    font-size: 12px;
    color: var(--island-text-primary, #ffffff);
    word-break: break-word;
    line-height: 1.35;
  }

  .task-title.strike {
    text-decoration: line-through;
    color: var(--island-text-muted, #94a3b8);
  }

  .task-meta {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 9.5px;
  }

  .priority-chip {
    font-weight: 700;
    color: var(--prio-color);
    letter-spacing: 0.5px;
  }

  .due-chip {
    color: var(--island-text-muted, #94a3b8);
    display: flex;
    align-items: center;
    gap: 3px;
    font-size: 10px;
    padding: 1px 6px;
    border-radius: 4px;
    background: rgba(255, 255, 255, 0.05);
  }

  .due-chip.is-today {
    color: #f59e0b;
    background: rgba(245, 158, 11, 0.12);
    border: 1px solid rgba(245, 158, 11, 0.25);
    font-weight: 500;
  }

  .due-chip.is-past {
    color: #f87171;
    background: rgba(239, 68, 68, 0.12);
    border: 1px solid rgba(239, 68, 68, 0.3);
    font-weight: 600;
  }

  .del-btn {
    width: 22px;
    height: 22px;
    border-radius: 50%;
    border: none;
    background: transparent;
    color: var(--island-text-muted, #94a3b8);
    font-size: 10px;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    opacity: 0.5;
    transition: all 0.15s ease;
  }

  .task-item:hover .del-btn {
    opacity: 1;
  }

  .del-btn:hover {
    background: rgba(239, 68, 68, 0.2);
    color: #ef4444;
  }

  .empty-todo-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 6px;
    padding: 40px 10px;
    color: var(--island-text-muted, #94a3b8);
    text-align: center;
  }

  .empty-icon {
    font-size: 32px;
    margin-bottom: 4px;
    opacity: 0.8;
  }

  .empty-title {
    font-size: 12px;
    max-width: 320px;
    line-height: 1.4;
  }

  .empty-hint {
    font-size: 10px;
    opacity: 0.65;
  }

  .empty-hint code {
    background: rgba(255, 255, 255, 0.1);
    padding: 2px 5px;
    border-radius: 4px;
  }

  @keyframes fadeIn {
    from { opacity: 0; transform: translateY(-3px); }
    to { opacity: 1; transform: translateY(0); }
  }
</style>
