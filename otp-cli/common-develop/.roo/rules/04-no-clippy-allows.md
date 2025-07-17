# Roo Instructions: Do Not Silence Clippy Lints

To maintain high code quality and consistency, Clippy lints must not be silenced with `#[allow(...)]` attributes. All developers, including AI assistants, are required to address the underlying issues identified by Clippy instead of bypassing them.

## Rationale

Clippy provides valuable suggestions for improving code correctness, performance, and readability. Silencing lints defeats the purpose of using it and can lead to technical debt, hidden bugs, and inconsistent coding styles.

## Policy

-   **No `#[allow(...)]` for Clippy Lints**: The use of `#[allow(clippy::...)]` is strictly prohibited in the codebase.
-   **Address the Root Cause**: Instead of silencing a lint, you must refactor the code to resolve the warning.
-   **Exceptions**: In the rare and exceptional case that a lint must be silenced, it requires explicit, documented approval in the corresponding pull request or task description. The `#[allow(...)]` attribute must be accompanied by a comment explaining the justification for the exception. This should only be considered when a Clippy lint is incorrect or when the suggested fix is not feasible or desirable for a well-justified reason.

By adhering to this policy, we ensure that our codebase remains clean, consistent, and of high quality.