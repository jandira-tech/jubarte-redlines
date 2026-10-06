# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
# SPDX-License-Identifier: AGPL-3.0-only
"""One scripted MCP session over stdio with jubarte-mcp.

Speaks JSON-RPC (newline-delimited, MCP stdio transport) to
    uvx --from "$JUBARTE_MCP_FROM" jubarte-mcp --root .
where JUBARTE_MCP_FROM defaults to 'jubarte-redlines[mcp]' (the release on
PyPI); CI sets it to the checkout ('jubarte-redlines[mcp] @ ./jubarte-python').
sends initialize, the initialized notification, tools/list, and a
tools/call that reads input.docx as text, and appends every message
sent or received to mcp_session.jsonl as one JSON object per line.

Standard library only.
"""

import json
import os
import select
import subprocess
import sys
from pathlib import Path

FOLDER = Path(__file__).resolve().parent
TIMEOUT = 300.0  # uvx may build the environment on first use
SOURCE = os.environ.get("JUBARTE_MCP_FROM", "jubarte-redlines[mcp]")

proc = subprocess.Popen(
    ["uvx", "--from", SOURCE, "jubarte-mcp", "--root", str(FOLDER)],
    stdin=subprocess.PIPE,
    stdout=subprocess.PIPE,
    stderr=open(FOLDER / "mcp_stderr.txt", "w"),
    text=True,
    bufsize=1,
)
log = open(FOLDER / "mcp_session.jsonl", "w")


def record(direction, message):
    log.write(json.dumps({"dir": direction, **message}, ensure_ascii=False) + "\n")
    log.flush()


def send(message):
    line = json.dumps(message, ensure_ascii=False)
    proc.stdin.write(line + "\n")
    proc.stdin.flush()
    record("send", message)


def receive():
    ready, _, _ = select.select([proc.stdout], [], [], TIMEOUT)
    if not ready:
        raise TimeoutError("no MCP response within %ss" % int(TIMEOUT))
    line = proc.stdout.readline()
    if not line:
        raise EOFError("MCP server closed stdout")
    message = json.loads(line)
    record("recv", message)
    return message


send({
    "jsonrpc": "2.0",
    "id": 1,
    "method": "initialize",
    "params": {
        "protocolVersion": "2025-06-18",
        "capabilities": {},
        "clientInfo": {"name": "adoption-evidence", "version": "0"},
    },
})
init = receive()
server = init.get("result", {}).get("serverInfo", {})
print("initialized: %s %s (protocol %s)" % (
    server.get("name"), server.get("version"),
    init.get("result", {}).get("protocolVersion")))

send({"jsonrpc": "2.0", "method": "notifications/initialized"})

send({"jsonrpc": "2.0", "id": 2, "method": "tools/list"})
tools = receive().get("result", {}).get("tools", [])
print("tools: %s" % ", ".join(t["name"] for t in tools))

name = "docx_text"
if name not in [t["name"] for t in tools]:
    sys.exit("server offers no docx_text tool: %s" % [t["name"] for t in tools])

send({
    "jsonrpc": "2.0",
    "id": 3,
    "method": "tools/call",
    "params": {"name": name, "arguments": {"path": "input.docx"}},
})
result = receive().get("result", {})
text = "".join(
    block.get("text", "") for block in result.get("content", [])
)
print("docx_text returned %d characters; first lines:" % len(text))
for line in text.splitlines()[:4]:
    print("  " + line)

proc.stdin.close()
proc.wait(timeout=TIMEOUT)
print("server exited %s" % proc.returncode)
