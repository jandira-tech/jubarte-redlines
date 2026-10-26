with open("jubarte-python/python/jubarte_redlines/mcp_server.py", "r") as f:
    content = f.read()

# Let's check where ToolError is imported.
import_search = "from mcp.server.mcpserver.exceptions import ToolError"
if import_search in content:
    print("ToolError is imported")
else:
    print("ToolError is NOT imported")
