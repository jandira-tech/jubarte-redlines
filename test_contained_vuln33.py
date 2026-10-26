import os
import pathlib

root = pathlib.Path("/tmp/root")

def contained(path: str) -> pathlib.Path:
    p = (root / pathlib.Path(path).expanduser()).resolve()
    if not p.is_relative_to(root):
        raise ValueError(f"{path} is outside root {root}")
    # Here is the issue: it returns the LEXICAL path without resolving!
    # "Keep the lexical path for capability-relative operations; resolving is only an early diagnostic, never the authorization for actual I/O."
    # Wait, the Rust code uses `cap_std` which prevents traversing `..` OUT of the open directory.
    # What if the user provides `..`?
    # e.g., `path` = "dir/../dir/file.txt".
    # resolve() -> `/tmp/root/dir/file.txt` (inside root).
    # `cap_std` allows traversing `..` as long as it doesn't escape the root.
    pass
