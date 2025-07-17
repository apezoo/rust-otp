# Roo Instructions: Enforce Clean Git State Before New Tasks

To ensure project stability and maintain a clean, atomic commit history, you must verify that the Git working directory is clean before starting any new task.

## Workflow

1.  **Initial Check**: At the very beginning of a new task, execute `git status --porcelain` to check the state of the repository.

2.  **Analyze the State**:
    *   **If the output is empty**: The repository is clean. You can proceed with the new task.
    *   **If the output is not empty**: The repository is "dirty," containing uncommitted changes. You **must not** proceed with the new task.

3.  **Action on Dirty Workspace**:
    If the workspace is dirty, you must deny the new task and guide the user to resolve the current state by recommending the following steps:
    *   **Investigate**: Review the current changes (`git diff HEAD`) to understand the state of the ongoing work.
    *   **Complete**: Finalize the in-progress task. (if the task was incomplete, investigate then continue...)
    *   **Test**: Run the necessary tests to validate the changes. For example: the Rust backend, use `cargo test --workspace`.
    *   **Commit**: Commit the completed work with a clear, conventional commit message.