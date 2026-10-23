## Summary

```diff
-        except Exception:
-            pass
+        except Exception:
+            logging.exception("Failed to write resolution text")
```

## Evidence

- **Before:**
  ```text
  jubarte-python/python/jubarte_redlines/__main__.py:661:9: try-except-pass: `try`-`except`-`pass` detected, consider logging the exception
  jubarte-python/python/jubarte_redlines/__main__.py:714:9: try-except-pass: `try`-`except`-`pass` detected, consider logging the exception
  ```
- **After:** `uvx ruff check --select S jubarte-python/python/jubarte_redlines/__main__.py` returns `All checks passed!`.

## Merge Danger

**Door:** two-way

**Blast Radius:** Isolated
