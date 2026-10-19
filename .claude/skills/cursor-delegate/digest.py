#!/usr/bin/env python3
"""Turn cursor-agent `--output-format stream-json` into a short digest for the supervising agent.

Reads events on stdin, prints one line per meaningful event (thinking deltas are dropped - they
cost the supervisor tokens and say nothing checkable). Side files, written next to the digest:
<base>.session (session id for --resume) and <base>.last.md (the agent's final message, in full).
Usage: python3 digest.py <base-path-without-extension>
"""
import json
import sys
import time

base = sys.argv[1]
args_by_call = {}


def out(tag, text=""):
    line = f"[{time.strftime('%H:%M:%S')}] {tag} {text}".rstrip()
    print(line, flush=True)


def one_line(text, limit):
    text = " / ".join(part.strip() for part in str(text).splitlines() if part.strip())
    return text if len(text) <= limit else text[: limit - 3] + "..."


def tool_target(args):
    for key in ("command", "path", "filePath", "targetFile", "pattern", "query", "url", "globPattern"):
        if isinstance(args, dict) and args.get(key):
            return str(args[key])
    return ""


for raw in sys.stdin:
    try:
        event = json.loads(raw)
    except ValueError:
        if raw.strip():
            out("RAW", one_line(raw, 300))
        continue
    kind, sub = event.get("type"), event.get("subtype")
    if kind == "system" and sub == "init":
        with open(base + ".session", "w") as f:
            f.write(event.get("session_id", "") + "\n")
        out("SESSION", f"{event.get('session_id')} model={event.get('model')}")
    elif kind == "assistant":
        text = "".join(c.get("text", "") for c in event.get("message", {}).get("content", []))
        if text.strip():
            out("SAY", one_line(text, 400))
    elif kind == "tool_call":
        call = event.get("tool_call", {})
        name = next(iter(call), "?")
        body = call.get(name, {})
        if sub == "started":
            args_by_call[event.get("call_id")] = body.get("args", {})
            continue
        result = body.get("result", {}) or {}
        status = next(iter(result), "?")
        detail = result.get(status) if isinstance(result.get(status), dict) else {}
        target = tool_target(args_by_call.pop(event.get("call_id"), {})) or tool_target(detail)
        reason = detail.get("reason") or detail.get("error") or ""
        tag = "SHELL" if name == "shellToolCall" else name.replace("ToolCall", "").upper()
        text = f"{status} {one_line(target, 200)}"
        if status in ("rejected", "error") and reason:
            text += " | " + one_line(reason, 200)
        out(tag, text)
    elif kind == "result":
        text = event.get("result", "") or ""
        with open(base + ".last.md", "w") as f:
            f.write(text + "\n")
        usage = event.get("usage") or {}
        out("DONE", f"{sub} {event.get('duration_ms', 0) // 1000}s in={usage.get('inputTokens')} "
                    f"out={usage.get('outputTokens')} cache={usage.get('cacheReadTokens')}")
        for line in text.splitlines():
            if line.strip().upper().startswith("QUESTION:"):
                out("QUESTION", line.strip()[len("QUESTION:"):].strip())
