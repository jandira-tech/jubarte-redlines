import subprocess
with open("scripts/security-check.sh", "r") as f:
    lines = f.readlines()
with open("scripts/security-check-no-tools.sh", "w") as f:
    for line in lines:
        if "command -v cargo-deny" in line or "command -v gitleaks" in line: continue
        f.write(line)
