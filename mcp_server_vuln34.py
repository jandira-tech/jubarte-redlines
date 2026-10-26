# What if the user provides `C:/abs.txt`?
# pathlib.Path("C:/abs.txt").expanduser() -> C:/abs.txt
# root / C:/abs.txt -> /tmp/root/C:/abs.txt
# resolve() -> /tmp/root/C:/abs.txt
# is_relative_to(root) -> True.
# returned: /tmp/root/C:/abs.txt
# publish gets files: `(root / "C:/abs.txt").relative_to(root).as_posix()` -> `C:/abs.txt`
# Rust `validate_relative` gets `C:/abs.txt`.
# In Rust `validate_relative("C:/abs.txt")`
# It splits by `/`. First part is `C:`.
# stem is `C:`.
# `["COM", "LPT"].iter().any(...)` is False.
# It doesn't contain any control characters or forbidden characters (backslash, colon). Wait, it contains a colon!
# `matches!(c, '\\' | ':' | '%' | ...)` -> `c == ':'`.
# So it is REJECTED by `validate_relative` because it contains `:`!

# What if path is `...`?
# stem is `...`
# no colon.
# split by `/`.
# returns `...`.
