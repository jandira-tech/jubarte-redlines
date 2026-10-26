with open("jubarte-python/python/jubarte_redlines/mcp_server.py", "r") as f:
    content = f.read()

# Add a check to `docx_edit`:
# if "operations" in plan and len(plan["operations"]) > 1000:
#     raise ToolError("Edit plan exceeds 1000 operations")
# if len(json.dumps(plan)) > 1_000_000:
#     raise ToolError("Edit plan exceeds 1MB limit")

search = """
        doc = load(path)
        out = contained(out_dir)
        try:
            result = engine(lambda: doc.edit(plan))
"""

replace = """
        if len(json.dumps(plan)) > 1024 * 1024:
            raise ToolError("Edit plan exceeds 1MB limit")

        doc = load(path)
        out = contained(out_dir)
        try:
            result = engine(lambda: doc.edit(plan))
"""

new_content = content.replace(search, replace)

search_resolve = """
    def resolve(
        action: str,
        path: str,
        out: str,
        ids: Sequence[str] | None,
        authors: Sequence[str] | None,
        kinds: Sequence[str] | None,
        overwrite: bool,
    ) -> dict[str, Any]:
        doc = load(path)
"""

replace_resolve = """
    def resolve(
        action: str,
        path: str,
        out: str,
        ids: Sequence[str] | None,
        authors: Sequence[str] | None,
        kinds: Sequence[str] | None,
        overwrite: bool,
    ) -> dict[str, Any]:
        if action not in ("accept", "reject"):
            raise ToolError(f"invalid action {action!r}")
        doc = load(path)
"""

new_content = new_content.replace(search_resolve, replace_resolve)

with open("jubarte-python/python/jubarte_redlines/mcp_server.py", "w") as f:
    f.write(new_content)

print("Patched mcp_server.py")
