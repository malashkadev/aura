### Task 2: Global Win32 Keyboard Hook

**Files:**
- Create: `d:/Загрузки/1/src-tauri/src/keyboard_hook.rs`
- Modify: `d:/Загрузки/1/src-tauri/src/main.rs`

- [ ] **Step 1: Implement Win32 Low-Level Keyboard Hook**
  Create `keyboard_hook.rs` containing hook logic using `SetWindowsHookExW` to capture down/up transitions of `N` while `Alt` is held. Define callback interfaces to notify Tauri core.

- [ ] **Step 2: Integrate hook in main.rs**
  Start the hook thread on Tauri app initialization and forward events to Tauri event bus.

---

