#!/bin/bash
# Claude Code PreToolUse hook (matcher Bash): blocks bypassing the commit-msg hook and adding agent
# authorship. The rule is AGENTS.md, "Git and releases"; wired in .claude/settings.json.
# Denies: --no-verify and its abbreviations on any git command; git commit -n (also inside clusters
# such as -anm); --author and its abbreviations on git commit; author overrides through
# user.name/user.email and GIT_AUTHOR_*/GIT_COMMITTER_*; core.hooksPath and include.path overrides
# (git -c, --config-env, git config, GIT_CONFIG_* in the environment or export); aliases that do the
# same; agent authorship lines in commit, commit-tree, tag, notes and merge.
# The command is parsed like a shell would (python3 shlex): quotes, ; && || | ( ) { }, if/then/do/!,
# line continuations, $(...) and `...` (also inside quotes), $'...', nested bash -c, eval, <<< and | sh.
# A quoted commit message is not an option. This catches agent mistakes, not deliberate evasion:
# editing .git/config, other interpreters (python -c) and variables ($V) are invisible to it.
# Result: exit 2 with the reason on stderr; Claude Code cancels the call and shows the reason.
# Any failure of the check itself (no python3, an exception) blocks commands containing git.
# Tests: bash .claude/hooks/tests/guard-git-test.sh .claude/hooks/guard-git.sh

read -r -d '' GUARD_PY <<'PY'
import json, os, re, shlex, sys

def deny(reason):
    sys.stderr.write("Blocked by .claude/hooks/guard-git.sh: " + reason + ". Agent or CLI authorship and "
                     "bypassing git hooks are forbidden (AGENTS.md, Git and releases). Commit with plain "
                     "git commit -m, no trailers, no --no-verify.\n")
    sys.exit(2)

AUTHOR_RE = re.compile(
    r"co-authored-by\s*[:=]|claude-session\s*[:=]|made[- ]with\s*:?\s*cursor"
    r"|(generated|made)[- ](with|by)\s*:?\s*\[?\s*(claude|cursor|gemini|codex|copilot|antigravity)"
    r"|cursoragent@cursor\.com|noreply@anthropic\.com")
BAD_ENV_RE = re.compile(
    r"^(GIT_CONFIG_(PARAMETERS|COUNT|KEY_\d+|VALUE_\d+|GLOBAL|SYSTEM)|GIT_(AUTHOR|COMMITTER)_(NAME|EMAIL))=", re.I)
PUNCT = "();<>|&\n"
KEYWORDS = {"!", "{", "}", "if", "then", "elif", "else", "fi", "do", "done", "while", "until"}
WRAPPERS = {"command", "exec", "nice", "nohup", "time", "timeout", "sudo", "env", "xargs", "stdbuf", "caffeinate"}
DECLARERS = {"export", "declare", "typeset", "local", "readonly"}
SHELLS = {"bash", "sh", "zsh", "dash", "ksh"}
GIT_GLOBAL_WITH_ARG = {"-c", "-C", "--git-dir", "--work-tree", "--namespace", "--config-env", "--attr-source"}
# git commit short options whose value is attached or the next word; u and S take only an attached value
COMMIT_SHORT_WITH_ARG = set("mFcCt")
COMMIT_SHORT_OPTIONAL = set("uS")
COMMIT_LONG_WITH_ARG = {"--message", "--file", "--author", "--date", "--trailer", "--template",
                        "--reuse-message", "--reedit-message", "--fixup", "--squash", "--cleanup",
                        "--pathspec-from-file"}
AUTHOR_TEXT_SUBS = {"commit", "commit-tree", "tag", "notes", "merge"}
def is_prefix_of(token, full, min_len):
    name = token.split("=", 1)[0]
    return len(name) >= min_len and full.startswith(name)

def split_commands(text):
    text = text.replace("\\\n", " ").replace("`", "\n")  # `...` is a separate command
    text = re.sub(r"\$(['\"])", r"\1", text)              # $'...' and $"..." are plain quotes
    lex = shlex.shlex(text, posix=True, punctuation_chars=PUNCT)
    lex.whitespace = " \t\r"
    lex.whitespace_split = True
    lex.commenters = ""
    cmds, cur = [], []
    for tok in lex:
        if tok and all(ch in PUNCT for ch in tok):
            if cur:
                cmds.append(cur)
                cur = []
        else:
            cur.append(tok)
    if cur:
        cmds.append(cur)
    return cmds

def check_option_token(t):
    # brace expansion is dangerous only in an option name (-{n,}, --no-veri{fy,}), not in a value (--format={%h,%s})
    if re.match(r"^-[A-Za-z-]*\{[^}]*,", t):
        deny("brace expansion in a git option")

def check_config_pair(key, value, depth):
    k = key.lower()
    if k == "core.hookspath" and value != ".githooks":  # .githooks is the documented setting
        deny("overriding core.hooksPath disables the commit-msg hook")
    if k.startswith("include.") or k.startswith("includeif."):
        deny("an included config can override core.hooksPath")
    if k in ("user.name", "user.email"):
        deny("changing user.name or user.email sets a foreign commit author")
    if k.startswith("alias.") and value:
        if value.startswith("!"):
            check_text(value[1:], depth + 1)
        else:
            check_text("git " + value, depth + 1)

def check_config_kv(kv, depth):
    key, _, value = kv.partition("=")
    check_config_pair(key, value, depth)

def check_commit(rest):
    i = 0
    while i < len(rest):
        t = rest[i]
        if t == "--":
            break
        check_option_token(t)
        if t.startswith("--"):
            if t.startswith("--no-") and is_prefix_of(t, "--no-verify", 9):
                deny("--no-verify bypasses the commit-msg hook")
            if is_prefix_of(t, "--author", 4):
                deny("--author sets a foreign author on the commit")
            if t in COMMIT_LONG_WITH_ARG:
                i += 2
                continue
        elif re.match(r"^-[A-Za-z]+$", t):
            for k, ch in enumerate(t[1:]):
                if ch == "n":
                    deny("git commit -n skips the commit-msg hook")
                if ch in COMMIT_SHORT_OPTIONAL:
                    break  # attached value only: the rest of the cluster
                if ch in COMMIT_SHORT_WITH_ARG:
                    if k == len(t) - 2:
                        i += 1  # the value is the next word
                    break
        i += 1

CONFIG_WITH_ARG = {"-f", "--file", "--blob", "-t", "--type", "--default", "--comment", "--value"}
CONFIG_READ_FLAGS = {"--get", "--get-all", "--get-regexp", "--list", "-l", "--get-color", "--get-colorbool",
                     "--get-urlmatch", "--show-origin", "--show-scope", "--name-only"}
CONFIG_READ_SUBS = {"get", "list", "get-color", "get-colorbool"}
CONFIG_WRITE_SUBS = {"set", "unset", "rename-section", "remove-section", "edit"}
CONFIG_WRITE_FLAGS = {"--add", "--replace-all", "--unset", "--unset-all", "--rename-section", "--remove-section",
                      "--edit", "-e"}

def parse_config(rest):
    # -> (subcommand or None, option names, positional words); option values are not words
    words, flags, i = [], set(), 0
    while i < len(rest):
        t = rest[i]
        if t == "--":
            words += rest[i + 1:]
            break
        if t in CONFIG_WITH_ARG:
            i += 2
            continue
        if t.startswith("-"):
            flags.add(t.split("=", 1)[0])
        else:
            words.append(t)
        i += 1
    sub = None
    if words and words[0] in CONFIG_READ_SUBS | CONFIG_WRITE_SUBS:
        sub, words = words[0], words[1:]
    return sub, flags, words

def config_is_read(sub, flags, words):
    if sub in CONFIG_READ_SUBS:
        return True
    if sub or flags & CONFIG_WRITE_FLAGS:
        return False
    return bool(flags & CONFIG_READ_FLAGS) or len(words) < 2  # --get, --list or a bare key

def check_config(rest, depth):
    sub, flags, words = parse_config(rest)
    if config_is_read(sub, flags, words) or sub == "edit" or flags & {"--edit", "-e"}:
        return  # an interactive edit is the documented gap, like editing .git/config
    key = words[0].lower() if words else ""
    if sub in ("rename-section", "remove-section") or flags & {"--rename-section", "--remove-section"}:
        if key in ("core", "user") or key.startswith("include"):
            deny("renaming or removing a config section that holds core.hooksPath or the author")
        return
    if sub == "unset" or flags & {"--unset", "--unset-all"}:
        if key == "core.hookspath" or key.startswith(("include.", "includeif.")):
            deny("unsetting core.hooksPath or an include changes which hooks run")
        return
    if words:
        check_config_pair(words[0], words[1] if len(words) > 1 else "", depth)

def check_git(assignments, args, depth):
    for a in assignments:
        if re.match(r"^GIT_CONFIG_(GLOBAL|SYSTEM)=/dev/null$", a, re.I):
            continue  # disabling the global config is safe
        if BAD_ENV_RE.match(a) or "core.hookspath" in a.lower():
            deny("git config or commit author overridden through the environment")
    j, sub = 0, None
    while j < len(args):
        a = args[j]
        check_option_token(a)
        if a in GIT_GLOBAL_WITH_ARG:
            if j + 1 < len(args) and a in ("-c", "--config-env"):
                check_config_kv(args[j + 1], depth)
            j += 2
            continue
        if a.startswith("-"):
            if a.startswith("--config-env="):
                check_config_kv(a.split("=", 1)[1], depth)
            if "core.hookspath" in a.lower():
                deny("overriding core.hooksPath disables the commit-msg hook")
            j += 1
            continue
        sub = a.lower()
        j += 1
        break
    rest = args[j:]
    if sub != "commit":  # check_commit parses commit options that take values
        for t in rest:
            if t == "--":
                break
            check_option_token(t)
            if t.startswith("--no-") and is_prefix_of(t, "--no-verify", 9):
                deny("--no-verify bypasses the commit-msg hook")
    if sub == "config":
        check_config(rest, depth)
    if sub == "commit":
        check_commit(rest)
    if sub in AUTHOR_TEXT_SUBS and AUTHOR_RE.search(" ".join(rest).lower()):
        deny("agent or CLI authorship in the commit text")

def check_command(tokens, depth):
    for t in tokens:  # $(...) and `...` inside a word, double quotes included
        if "$(" in t:
            check_text(t[t.index("$(") + 2:], depth + 1)
        if "\n" in t:
            check_text(t, depth + 1)
    i, assignments = 0, []
    while i < len(tokens):
        t = tokens[i]
        base = os.path.basename(t).lower()
        if re.match(r"^[A-Za-z_][A-Za-z0-9_]*=", t):
            assignments.append(t)
            i += 1
        elif t in KEYWORDS:
            i += 1
        elif base in WRAPPERS:
            i += 1
            while i < len(tokens) and (tokens[i].startswith("-") or re.match(r"^\d+[smhd]?$", tokens[i])):
                i += 1
        else:
            break
    if i >= len(tokens):
        return False
    first = tokens[i]
    if re.search(r"\s", first):  # a whole command in one word: bash <<< "git ...", "(git ...)"
        check_text(first, depth + 1)
        return False
    prog = os.path.basename(first.lstrip("`$({!")).lower()
    args = tokens[i + 1:]
    if prog in DECLARERS:
        check_git([a for a in args if "=" in a], [], depth)
    elif prog == "eval":
        check_text(" ".join(args), depth + 1)
    elif prog in SHELLS:
        for k, a in enumerate(args):
            if re.match(r"^-[A-Za-z]*c[A-Za-z]*$", a) and k + 1 < len(args):
                check_text(args[k + 1], depth + 1)
                return False
        return True  # the shell reads commands from stdin
    elif prog == "git":
        check_git(assignments, args, depth)
    return False

HEREDOC_RE = re.compile(r"(?<!<)<<(?!<)-?[ \t]*(['\"]?)([A-Za-z_][A-Za-z0-9_]*)\1")

def strip_heredocs(text):
    # Heredoc bodies are data, not commands: cut them from parsing and return them separately
    out, bodies, lines, i = [], [], text.split("\n"), 0
    while i < len(lines):
        line = lines[i]
        out.append(line)
        i += 1
        for m in HEREDOC_RE.finditer(line):
            body = []
            while i < len(lines) and lines[i].strip() != m.group(2):
                body.append(lines[i])
                i += 1
            i += 1
            bodies.append("\n".join(body))
    return "\n".join(out), bodies

def raw_check(text):
    # Coarse check when parsing fails or the command is too long
    low = text.lower()
    if re.search(r"(^|[^\w./-])git\s", low) and ("--no-veri" in low or "core.hookspath" in low):
        deny("unparsable command with a git-hook bypass flag")

def parse(text):
    try:
        return split_commands(text)
    except ValueError:
        pass
    try:  # an apostrophe in a # comment: retry without comments
        return split_commands(re.sub(r"(^|[ \t;&|])#[^\n]*", r"\1", text))
    except ValueError:
        return None

def check_text(text, depth=0):
    if depth > 6:
        return
    text, bodies = strip_heredocs(text)
    if bodies and re.search(r"(^|[^\w./-])git\s[^\n]*\b(commit|commit-tree|tag|notes|merge)\b", text.lower()) \
            and any(AUTHOR_RE.search(b.lower()) for b in bodies):
        deny("agent or CLI authorship in the commit text")
    cmds = parse(text)
    if cmds is None:
        raw_check(text)
        return
    stdin_shell = False
    for c in cmds:
        stdin_shell = check_command(c, depth) or stdin_shell
    if stdin_shell:  # echo "git ..." | sh: check the strings that reach the shell
        for c in cmds:
            for t in c:
                if re.search(r"\s", t) and "git" in t.lower():
                    check_text(t, depth + 1)

try:
    data = sys.stdin.buffer.read().decode("utf-8", "replace")
    command = (json.loads(data).get("tool_input") or {}).get("command") or ""
except Exception:
    sys.exit(0)
command = command if isinstance(command, str) else str(command)
if "git" in command.lower():
    if len(command) > 200000:  # parsing this command would not fit the hook timeout
        raw_check(command)
    else:
        check_text(command)
sys.exit(0)
PY

input=$(cat)
py=/usr/bin/python3
[ -x "$py" ] || py=$(command -v python3)
if [ -n "$py" ]; then
    printf '%s' "$input" | "$py" -c "$GUARD_PY"
    rc=$?
    [ $rc -eq 0 ] || [ $rc -eq 2 ] && exit $rc
fi
case "$(printf '%s' "$input" | tr '[:upper:]' '[:lower:]')" in
    *git*) echo "Blocked by .claude/hooks/guard-git.sh: the check itself failed (python3 missing or crashed), a git command cannot be verified." >&2
           exit 2 ;;
esac
exit 0
