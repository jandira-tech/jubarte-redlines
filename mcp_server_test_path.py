from pathlib import Path
root = Path('/tmp/mcp_root').resolve()
root.mkdir(parents=True, exist_ok=True)
path_str = "/tmp/mcp_root/../../etc/passwd"
print(Path(path_str).expanduser())
