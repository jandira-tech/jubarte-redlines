from pathlib import Path

root = Path('/tmp/mcp_root').resolve()
root.mkdir(parents=True, exist_ok=True)

path_str = "/etc/passwd"
# Simulation of what `contained` does:
expanded = Path(path_str).expanduser()
print(f"expanded: {expanded}")
joined = root / expanded
print(f"joined: {joined}")
resolved = joined.resolve()
print(f"resolved: {resolved}")
print(f"is relative? {resolved.is_relative_to(root)}")
