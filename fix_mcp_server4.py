with open("jubarte-python/python/jubarte_redlines/mcp_server.py", "r") as f:
    content = f.read()

# Change it to "outside root directory" so that "outside root" matches!
content = content.replace('raise ToolError("path is outside server root")', 'raise ToolError("path is outside root directory")')

with open("jubarte-python/python/jubarte_redlines/mcp_server.py", "w") as f:
    f.write(content)
