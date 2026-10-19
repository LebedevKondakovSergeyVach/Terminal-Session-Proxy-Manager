#!/bin/bash
# beforeShellExecution guard for Cursor agents started headless by skill cursor-delegate.
# Active only when CURSOR_DELEGATE=1 - the user's own Cursor sessions pass through untouched.
# Nobody can approve a prompt in a headless run, so everything AGENTS.md reserves for an explicit user
# request is denied outright: git mutations, GitHub writes and the raw API, publishing and installing,
# recursive rm. The command is split like a shell would (python3 shlex): quotes, ; && || | ( ) { },
# $(...) and `...`, nested bash -c and eval, wrappers such as env or /usr/bin/ paths. Heredoc bodies
# are data. Any failure of the check itself denies (fail-closed).
# Tests: bash .cursor/hooks/tests/delegate-guard-test.sh .cursor/hooks/delegate-guard.sh
# The user-level ~/.cursor/hooks/guard-git.sh, when installed, runs as well; any deny wins.

input=$(cat)
[ "${CURSOR_DELEGATE:-}" = 1 ] || { printf '{"permission":"allow"}\n'; exit 0; }

read -r -d '' GUARD_PY <<'PY'
import json, os, re, shlex, sys

def answer(permission, reason=""):
    out = {"permission": permission}
    if reason:
        out["user_message"] = "delegate-guard: " + reason
        out["agent_message"] = ("Blocked by .cursor/hooks/delegate-guard.sh: " + reason + ". You run headless "
            "for a parent agent: git mutations, GitHub writes, publishing and recursive rm are not yours to "
            "run (AGENTS.md). Do not retry it in another form; if the task needs it, stop and end your turn "
            "with a line starting QUESTION:.")
    print(json.dumps(out))
    sys.exit(0)

class Deny(Exception):
    pass

PUNCT = "();<>|&\n"
KEYWORDS = {"!", "{", "}", "if", "then", "elif", "else", "fi", "do", "done", "while", "until", "time"}
WRAPPERS = {"command", "exec", "nice", "nohup", "timeout", "sudo", "env", "xargs", "stdbuf", "caffeinate"}
WRAPPER_WITH_ARG = {"-u", "-g", "-n", "-s", "-C", "-I", "-P", "-L", "-k", "--user", "--group", "--chdir",
                    "--unset", "--signal", "--kill-after"}
SHELLS = {"bash", "sh", "zsh", "dash", "ksh"}
GIT_GLOBAL_WITH_ARG = {"-C", "-c", "--git-dir", "--work-tree", "--namespace", "--config-env",
                       "--attr-source", "--exec-path", "--super-prefix"}
GIT_MUTATING = {"commit", "commit-tree", "push", "pull", "fetch", "checkout", "switch", "restore", "merge",
                "rebase", "reset", "revert", "cherry-pick", "am", "apply", "clean", "rm", "mv", "add",
                "update-index", "update-ref", "symbolic-ref", "read-tree", "checkout-index", "submodule",
                "init", "clone", "gc", "prune", "prune-packed", "repack", "pack-refs", "notes", "filter-branch",
                "replace", "fast-import", "bisect", "sparse-checkout", "maintenance", "lfs", "send-pack",
                "fetch-pack", "receive-pack"}
TAG_READS = {"-l", "--list", "--contains", "--no-contains", "--merged", "--no-merged", "--points-at"}
CONFIG_WITH_ARG = {"-f", "--file", "--blob", "-t", "--type", "--default", "--comment", "--value"}
CONFIG_READ_FLAGS = {"--get", "--get-all", "--get-regexp", "--list", "-l", "--get-color", "--get-colorbool",
                     "--get-urlmatch", "--show-origin", "--show-scope", "--name-only"}
CONFIG_READ_SUBS = {"get", "list", "get-color", "get-colorbool"}
CONFIG_WRITE_SUBS = {"set", "unset", "rename-section", "remove-section", "edit"}
CONFIG_WRITE_FLAGS = {"--add", "--replace-all", "--unset", "--unset-all", "--rename-section", "--remove-section",
                      "--edit", "-e"}
BRANCH_READ_FLAGS = {"--show-current", "-a", "--all", "-r", "--remotes", "-v", "-vv", "--verbose", "--list",
                     "-l", "--no-color", "--column", "--no-column"}
BRANCH_FILTERS = {"--contains", "--no-contains", "--merged", "--no-merged", "--points-at", "--list", "-l"}
GH_WITH_ARG = {"-R", "--repo", "--hostname"}
GH_WRITES = {
    "pr": {"create", "edit", "merge", "close", "reopen", "comment", "review", "ready", "checkout", "lock", "unlock", "update-branch", "revert"},
    "issue": {"create", "edit", "close", "reopen", "comment", "delete", "transfer", "lock", "unlock", "pin", "unpin", "develop"},
    "release": {"create", "edit", "delete", "delete-asset", "upload"},
    "run": {"rerun", "cancel", "delete"},
    "workflow": {"run", "enable", "disable"},
    "repo": {"create", "edit", "delete", "rename", "archive", "unarchive", "fork", "sync", "deploy-key", "set-default", "clone"},
    "cache": {"delete"},
    "gist": {"create", "edit", "delete", "clone"},
}
GH_ALWAYS = {"api", "secret", "variable", "auth", "label", "ruleset", "extension", "alias", "config", "codespace"}
PUBLISH = {"cargo": {"publish", "yank", "owner", "login", "logout", "install", "uninstall"},
           "npm": {"publish", "unpublish", "login", "logout", "adduser", "deprecate", "dist-tag", "owner", "access"}}

def split_commands(text):
    text = text.replace("\\\n", " ").replace("`", "\n")
    text = re.sub(r"\$(['\"])", r"\1", text)
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

HEREDOC_RE = re.compile(r"(?<!<)<<(?!<)-?[ \t]*(['\"]?)([A-Za-z_][A-Za-z0-9_]*)\1")

SHELL_HEREDOC_RE = re.compile(r"(^|[;&|(\s/])(bash|sh|zsh|dash|ksh)(\s[^<]*)?<<")

def strip_heredocs(text):
    # Heredoc bodies are data, except when a shell reads them as its script.
    out, shell_bodies, lines, i = [], [], text.split("\n"), 0
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
            if SHELL_HEREDOC_RE.search(line):
                shell_bodies.append("\n".join(body))
    return "\n".join(out), shell_bodies

def config_is_read(rest):
    words, flags, i = [], set(), 0
    while i < len(rest):
        a = rest[i]
        if a == "--":
            words += rest[i + 1:]
            break
        if a in CONFIG_WITH_ARG:
            i += 2
            continue
        if a.startswith("-"):
            flags.add(a.split("=", 1)[0])
        else:
            words.append(a)
        i += 1
    if words and words[0] in CONFIG_READ_SUBS:
        return True
    if (words and words[0] in CONFIG_WRITE_SUBS) or flags & CONFIG_WRITE_FLAGS:
        return False
    return bool(flags & CONFIG_READ_FLAGS) or len(words) < 2

def check_git(args, depth):
    j = 0
    while j < len(args):
        a = args[j]
        if a in GIT_GLOBAL_WITH_ARG:
            if a == "-c" and j + 1 < len(args):
                key, _, value = args[j + 1].partition("=")
                if key.lower().startswith("alias.") and value:
                    check_text(value[1:] if value.startswith("!") else "git " + value, depth + 1)
            j += 2
            continue
        if a.startswith("-"):
            j += 1
            continue
        break
    if j >= len(args):
        return
    sub, rest = args[j].lower(), args[j + 1:]
    if sub in GIT_MUTATING:
        raise Deny("git " + sub + " changes the repository")
    if (sub == "hash-object" and "-w" in rest) or (sub == "reflog" and rest[:1] in (["delete"], ["expire"])) \
            or (sub in ("commit-graph", "multi-pack-index") and "write" in rest):
        raise Deny("git " + sub + " writes objects or refs")
    if sub == "remote" and rest and rest[0] not in ("-v", "--verbose", "show", "get-url"):
        raise Deny("git remote " + rest[0] + " changes remotes")
    if sub == "tag" and rest and not set(rest) & TAG_READS and not rest[0].startswith("-n"):
        raise Deny("git tag changes tags")
    if sub == "worktree" and rest[:1] != ["list"]:
        raise Deny("git worktree changes worktrees")
    if sub == "stash" and (not rest or rest[0] not in ("list", "show")):
        raise Deny("git stash changes the working tree")
    if sub == "branch":
        filtered = bool(set(rest) & BRANCH_FILTERS)
        for a in rest:
            if a.split("=", 1)[0] in BRANCH_READ_FLAGS or a.startswith(("--sort=", "--format=", "--color")):
                continue
            if a in BRANCH_FILTERS:
                continue
            if a.startswith("-") or not filtered:
                raise Deny("git branch " + a + " changes refs")
    if sub == "config" and not config_is_read(rest):
        raise Deny("git config writes configuration")

def check_gh(args):
    j = 0
    while j < len(args):
        a = args[j]
        if a in GH_WITH_ARG:
            j += 2
            continue
        if a.startswith("-"):
            j += 1
            continue
        break
    if j >= len(args):
        return
    group, action, k = args[j], "", j + 1
    while k < len(args):
        if args[k] in GH_WITH_ARG:
            k += 2
            continue
        if not args[k].startswith("-"):
            action = args[k]
            break
        k += 1
    if group in GH_ALWAYS or action in GH_WRITES.get(group, ()):
        raise Deny("gh " + group + (" " + action if action else "") + " writes to GitHub")

def check_rm(args):
    for a in args:
        if a == "--":
            break
        if a == "--recursive" or re.match(r"^-[A-Za-z]*[rR]", a):
            raise Deny("recursive rm")

def check_command(tokens, depth):
    for t in tokens:
        if "$(" in t:
            check_text(t[t.index("$(") + 2:], depth + 1)
        if "\n" in t:
            check_text(t, depth + 1)
    i = 0
    while i < len(tokens):
        t = tokens[i]
        base = os.path.basename(t).lower()
        if re.match(r"^[A-Za-z_][A-Za-z0-9_]*=", t) or t in KEYWORDS:
            i += 1
        elif base in WRAPPERS:
            i += 1
            while i < len(tokens) and (tokens[i].startswith("-") or re.match(r"^\d+[smhd]?$", tokens[i])
                                       or re.match(r"^[A-Za-z_][A-Za-z0-9_]*=", tokens[i])):
                i += 2 if tokens[i] in WRAPPER_WITH_ARG else 1
        else:
            break
    if i >= len(tokens):
        return False
    first = tokens[i]
    if re.search(r"\s", first):
        check_text(first, depth + 1)
        return False
    prog = os.path.basename(first.lstrip("$({!")).lower()
    args = tokens[i + 1:]
    if prog == "eval":
        check_text(" ".join(args), depth + 1)
    elif prog in SHELLS:
        for k, a in enumerate(args):
            if re.match(r"^-[A-Za-z]*c[A-Za-z]*$", a) and k + 1 < len(args):
                check_text(args[k + 1], depth + 1)
                return False
        # With a script argument the shell runs that file; without one it reads stdin.
        return all(a.startswith("-") for a in args)
    elif prog == "git":
        check_git(args, depth)
    elif prog == "gh":
        check_gh(args)
    elif prog == "rm":
        check_rm(args)
    elif prog in PUBLISH and "--dry-run" not in args:
        before = args[:args.index("--")] if "--" in args else args
        hit = next((a for a in before if a in PUBLISH[prog]), None)  # after +toolchain or global flags too
        if hit:
            raise Deny(prog + " " + hit + " publishes or installs")
    return False

def check_text(text, depth=0):
    if depth > 6:
        raise Deny("command nested too deeply to check")
    text, shell_bodies = strip_heredocs(text)
    for body in shell_bodies:
        check_text(body, depth + 1)
    cmds = split_commands(text)
    stdin_shell = False
    for c in cmds:
        stdin_shell = check_command(c, depth) or stdin_shell
    if stdin_shell:  # echo git push | sh: check what the other commands may feed the shell
        for c in cmds:
            if os.path.basename(c[0]).lower() not in SHELLS:
                check_text(" ".join(c[1:]).replace("\\n", "\n"), depth + 1)  # printf-style \n too

try:
    command = json.loads(sys.stdin.buffer.read().decode("utf-8", "replace")).get("command")
    if not isinstance(command, str):
        answer("deny", "the hook payload has no command string")
    check_text(command)
except Deny as e:
    answer("deny", str(e))
except Exception as e:
    answer("deny", "the command could not be checked (" + type(e).__name__ + ")")
answer("allow")
PY

py=/usr/bin/python3
[ -x "$py" ] || py=$(command -v python3)
if [ -n "$py" ] && out=$(printf '%s' "$input" | "$py" -c "$GUARD_PY") && [ -n "$out" ]; then
    printf '%s\n' "$out"
else
    printf '%s\n' '{"permission":"deny","user_message":"delegate-guard: the check itself failed","agent_message":"Blocked by .cursor/hooks/delegate-guard.sh: the check itself failed (python3 missing or crashed). Stop and end your turn with a line starting QUESTION:."}'
fi
exit 0
