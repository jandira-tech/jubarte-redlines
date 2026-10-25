from pathlib import Path

root = Path('/tmp/mcp_root').resolve()
root.mkdir(parents=True, exist_ok=True)
Path('/tmp/mcp_root/link').symlink_to('/etc/passwd')
path = root / 'link'

try:
    target = path.resolve()
    print(f"Target: {target}")
    print(f"Relative to root: {target.is_relative_to(root)}")
except Exception as e:
    print(e)
