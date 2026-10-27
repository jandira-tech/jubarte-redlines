with open("jubarte-python/tests/test_mcp_server.py", "r") as f:
    content = f.read()

# Since we changed "outside root {root}" to "path is outside server root", and "outside root or unsafe..." to "outside root or unsafe...", "outside root" should still be present in the error string!
# Wait!
# The log says:
# E       AssertionError: assert 'outside root' in 'Error executing tool docx_text: path is outside server root'
# Wait! 'outside root' IS in 'Error executing tool docx_text: path is outside server root' !!!
# Why did it fail???
# Let me look closely:
# AssertionError: assert 'outside root' in 'Error executing tool docx_text: path is outside server root'

# AH! In my patch:
# `raise ToolError("path is outside server root")`
# 'outside root' is in 'path is outside server root'
# Wait, why did the assert fail?
