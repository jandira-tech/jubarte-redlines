## 2026-11-09 - Avoid Experimental Features Unrelated To Vulnerabilities
**Vulnerability:** N/A
**Learning:** Fixing a build error due to `if let` syntax changes inside a file is an enhancement or general bugfix, not a security vulnerability fix. Security tasks must strictly adhere to identifying real vulnerabilities or adding explicit security measures, otherwise they should abort or perform actual security hardening.
**Prevention:** Always verify that the found issue relates directly to the listed vulnerability categories before attempting a fix.
