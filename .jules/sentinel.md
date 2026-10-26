## 2024-10-23 - Info Leakage in MCP Server ToolErrors
**Vulnerability:** MCP Server tools in Python exposed the server's absolute directory structure (the `root` directory) and system-level OS errors by directly passing absolute file paths into `ToolError` exception strings.
**Learning:** Even though internal `Path` resolution and the `cap-std` Rust backend securely prevented Path Traversal, printing those internally-used, unresolved absolute paths in error messages caused a CWE-209 Information Disclosure vulnerability.
**Prevention:** Avoid interpolating absolute file paths or raw stringified OS exceptions into user-facing exception messages (e.g. `ToolError`) that are returned to API consumers or LLM agents. Use generic, sanitized messages like `"path is outside server root"`.
