from pathlib import Path
import os

root = Path('/tmp/mcp_root').resolve()
root.mkdir(parents=True, exist_ok=True)
os.makedirs('/tmp/mcp_root/etc/passwd', exist_ok=True)
# If root is /tmp
root = Path('/tmp').resolve()

path_str = "/tmp/foo"
expanded = Path(path_str).expanduser()
joined = root / expanded
resolved = joined.resolve()
print(f"resolved: {resolved}")
print(f"is relative? {resolved.is_relative_to(root)}")
