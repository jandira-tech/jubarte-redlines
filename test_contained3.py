from pathlib import Path

root = Path('/tmp/mcp_root').resolve()
clean_path = "/tmp/mcp_root/../../etc/passwd".lstrip('/')
p = (root / Path(clean_path)).resolve()
print(p)
