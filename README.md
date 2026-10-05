# k7code - Rust GPUI Agent Workspace

`k7code` is a lightweight, pure-Rust desktop implementation inspired by the architecture and local features of [T3 Code](https://github.com/pingdotgg/t3code), built with [GPUI Kit](https://gpui-kit.com/) (`gpui-kit` / `gpui-component`).

## 1. Architectural Highlights

- **Pure Rust Stack**: Eliminates Electron, Node.js runtime, TypeScript compile step, and heavy memory overhead.
- **Native GPU Acceleration**: Renders at 120fps+ via GPUI's DirectX / Vulkan / Metal pipeline.
- **Clean Local Architecture**:
  - **Data Models** (`src/model.rs`): Projects, Threads, Turns, ToolSteps, Checkpoints, Diff Summaries, Runtime Modes, and Tool Approval Policies.
  - **Persistent Local Storage** (`src/storage.rs`): Stores threads and configurations in `~/.k7code/data.json` with automatic state recovery and initial repo setup.
  - **Git Checkpoint Engine** (`src/git.rs`): Computes head SHAs, branch status, numstat file changes, unified diffs, and reversible turn snapshots.
  - **Provider Execution Engine** (`src/provider/`):
    - `detector.rs`: Detects local CLI binaries (`claude`, `codex`, `cursor`, `grok`, `opencode`, `antigravity`, `ollama`) and queries version outputs.
    - `runner.rs`: Asynchronous execution harness with streaming deltas, tool proposal lifecycles, and approval gatekeeper (`RunnerCommand::ApproveTool` / `RejectTool`).
  - **GPUI Kit Desktop UI** (`src/ui/` & `src/main.rs`):
    - `HeaderView`: Thread title, running badge, git branch indicator, runtime mode switcher, approval policy switcher, diff panel toggle.
    - `SidebarView`: Project switcher, New Thread action, Active / Pinned / Archived filter tabs, thread action buttons (pin, archive, delete), provider counter, and settings modal trigger.
    - `ChatView`: Starter action pills, user bubbles, collapsible tool execution cards with stdout/stderr preview and approval actions, and git checkpoint revert banner.
    - `ComposerView`: Native multiline input, model selector button, and run/cancel controls.
    - `DiffPanelView`: Changed files list with additions/deletions, unified diff viewer, and discard changes button.
    - `SettingsModalView`: Detected CLI provider status table, default approval policy, and Ollama endpoint config.
