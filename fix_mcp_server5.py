with open("jubarte-python/python/jubarte_redlines/mcp_server.py", "r") as f:
    content = f.read()

# Change it back to include {path} but without {root} to pass the test `assert "nope.docx" in error_text(missing)`
# Wait! "nope.docx" is the {path}.
# So I should use `raise ToolError(f"cannot read file {path}: ...")`

content = content.replace('raise ToolError(f"cannot read file: {e.strerror or \'OS error\'}")', 'raise ToolError(f"cannot read file {path}: {e.strerror or \'OS error\'}")')

with open("jubarte-python/python/jubarte_redlines/mcp_server.py", "w") as f:
    f.write(content)
