from pathlib import Path

root = Path('/tmp/mcp_root').resolve()
root.mkdir(parents=True, exist_ok=True)

path = "/etc/passwd"
# The current implementation:
p1 = (root / Path(path).expanduser()).resolve()
print(f"Current: {p1} - relative to root? {p1.is_relative_to(root)}")

# What happens if we make it relative first by lstripping '/'
clean_path = path.lstrip('/')
p2 = (root / Path(clean_path).expanduser()).resolve()
print(f"Cleaned: {p2} - relative to root? {p2.is_relative_to(root)}")
