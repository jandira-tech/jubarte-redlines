# `action` is passed to `resolve` from `docx_accept` and `docx_reject`.
# The only calls are `resolve("accept", ...)` and `resolve("reject", ...)`
# So action is not user controlled directly.

# What about input bounding?
# Does MCP tool allow huge plans?
# "Add a check to prevent DoS by limiting the number of operations in docx_edit and limiting the length of strings in the plan."
# Actually, the user asked for input validation/sanitization in the Python `Document` or MCP server tool inputs. Let's see what Pyre or Ruff would flag.
# "Flag direct assignment of untrusted strings to element.innerHTML" - N/A
# "Flag hand-rolled isPrivateOrLocalHost hostname split checks" - N/A
# "Flag filename/path sanitizers that allow . characters without rejecting .. traversal segments before os.path.join / path.join."
# wait, `contained` allows `..`?
