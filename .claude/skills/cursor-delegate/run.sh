#!/usr/bin/env bash
# Starts or resumes a headless Cursor Grok 4.7 agent on a brief and writes its logs next to the brief.
#   bash run.sh <brief.md> [--effort E] [--context C] [--fast] [--mode plan|ask] [--strict]   start
#   bash run.sh <brief.md> --answer "<text>"                                                  resume the same session
#   bash run.sh --models                                                                      live Grok 4.7 ids
# Defaults are the SESSION.md model policy: --effort xhigh, --context 256k, Fast off. Model id passed to
# cursor-agent: grok-4.7[context=C,reasoning_effort=E,fast=true|false]. Brief: .ai/briefs/NN-<slug>.md
# (git-ignored). Logs: .ai/briefs/logs-NN/NN-<slug>.{jsonl,log,session,last.md,args,expect}. bash 3.2 compatible.
# Never passes --force: .cursor/hooks/delegate-guard.sh (CURSOR_DELEGATE=1) denies git mutations, GitHub
# writes, publishing and rm -r.
set -u
cd "$(git rev-parse --show-toplevel)" || exit 1
here=.claude/skills/cursor-delegate

# Model ids come from the live account list - they change with Cursor releases, never hard-code them.
list_models() { cursor-agent --list-models 2>/dev/null | perl -pe 's/\e\[[0-9;]*m//g' | sed -n 's/^\([A-Za-z0-9._-]*\) - \(.*\)$/\1 - \2/p'; }
if [ "${1:-}" = --models ]; then
    list_models | grep '^grok-4\.7-'
    exit 0
fi

usage="usage: bash $here/run.sh <brief.md> [--effort low|medium|high|xhigh] [--context 256k|500k] [--fast] [--mode plan|ask] [--strict] | --answer TEXT"
brief=${1:-}
[ -f "$brief" ] || { echo "$usage" >&2; exit 2; }
shift
# Anything but the defaults only on the user's explicit request.
effort=xhigh; context=256k; fast=false; mode=""; review="--auto-review"; answer=""; picked=""
need() { case "${2:-}" in ''|--*) echo "$1 needs a value" >&2; echo "$usage" >&2; exit 2 ;; esac; }
while [ $# -gt 0 ]; do
    case "$1" in
        --effort) need "$@"; effort=$2; picked=1; shift 2 ;;
        --context) need "$@"; context=$2; picked=1; shift 2 ;;
        --fast) fast=true; picked=1; shift ;;
        --mode) need "$@"; mode=$2; picked=1; shift 2 ;;
        --strict) review=""; picked=1; shift ;;
        --answer) need "$@"; answer=$2; shift 2 ;;
        --model) echo "--model is gone: only Grok 4.7 runs here; set --effort, --context, --fast" >&2; exit 2 ;;
        *) echo "unknown option $1" >&2; echo "$usage" >&2; exit 2 ;;
    esac
done

dir=$(dirname "$brief"); name=$(basename "$brief" .md)
nn=$(printf '%s' "$name" | sed -E 's/^([0-9]+).*/\1/')
logs="$dir/logs-$nn"; base="$logs/$name"

if [ -n "$answer" ]; then
    [ -z "$picked" ] || { echo "--answer reuses the start's model and flags ($base.args); drop the other options" >&2; exit 2; }
    session=$(head -1 "$base.session" 2>/dev/null)
    [ -n "$session" ] || { echo "no session in $base.session - start the run first" >&2; exit 2; }
    # A start round has the brief name in its argv, an answer round the session id.
    pgrep -f "cursor-agent -p.*($session|$name\.md)" >/dev/null && { echo "a round of $name is still running - wait for it" >&2; exit 2; }
    flags=$(cat "$base.args")
    prompt=$answer
    echo "=== resume $session $(date '+%d.%m.%Y %H:%M:%S')" >> "$base.log"
    set -- --resume "$session"
else
    # A start that died before the init event (rejected model, auth) leaves no .session and may be repeated.
    [ -s "$base.session" ] && { echo "$base.session exists - use --answer to continue, or a new brief number" >&2; exit 2; }
    case "$effort" in low) label=Low ;; medium) label=Medium ;; high) label=High ;; xhigh) label="Extra High" ;;
        *) echo "--effort must be low, medium, high or xhigh" >&2; exit 2 ;; esac
    case "$context" in 256k|500k) ;; *) echo "--context must be 256k or 500k" >&2; exit 2 ;; esac
    case "$mode" in ""|plan|ask) ;; *) echo "--mode must be plan or ask" >&2; exit 2 ;; esac
    # The flat id grok-4.7-<effort> must still be listed: a renamed or removed Grok family fails here, not mid-run.
    list_models | cut -d' ' -f1 | grep -qxF -- "grok-4.7-$effort" || {
        echo "grok-4.7-$effort is not in cursor-agent --list-models; Grok ids now:" >&2; list_models | grep -i grok >&2; exit 2; }
    model="grok-4.7[context=$context,reasoning_effort=$effort,fast=$fast]"
    expect="Grok 4.7 $(printf '%s' "$context" | tr k K) $label$( [ "$fast" = true ] && printf ' Fast')"
    mkdir -p "$logs"
    flags="--model $model $review${mode:+ --mode $mode}"
    printf '%s\n' "$flags" > "$base.args"
    printf '%s\n' "$expect" > "$base.expect"
    prompt="Read the brief $brief and carry it out exactly; it overrides your defaults where stricter.
Write the result to $dir/$name.result.md as the brief says; ask and plan modes allow writing markdown files.
You run headless: nobody can approve a prompt, rejected commands stay rejected - do not look for workarounds.
If you need a decision you cannot take from the brief, AGENTS.md or the code, stop and make the last line
of your final message: QUESTION: <one concrete question, with options if there are any>."
    echo "=== start $(date '+%d.%m.%Y %H:%M:%S') $flags" > "$base.log"
    set --
fi

git_state() { printf '%s %s\n' "$(git branch --show-current 2>/dev/null)" "$(git rev-parse --short HEAD 2>/dev/null)"; }
before=$(git_state)

# shellcheck disable=SC2086 - $flags is a word list on purpose; set -f keeps the bracketed model id from globbing
set -f
CURSOR_DELEGATE=1 cursor-agent -p --trust --output-format stream-json $flags "$@" "$prompt" 2>&1 \
    | tee -a "$base.jsonl" | python3 "$here/digest.py" "$base" | tee -a "$base.log"
code=${PIPESTATUS[0]}
set +f

after=$(git_state)
if [ "$before" != "$after" ]; then
    echo "[WARN] git branch or HEAD changed during the run: '$before' -> '$after'" | tee -a "$base.log"
fi
# The init event names the model that actually ran, context and Fast included: compare it with the request.
if [ -f "$base.expect" ]; then
    want=$(cat "$base.expect")
    got=$(sed -n 's/^\[[0-9:]*\] SESSION [^ ]* model=//p' "$base.log" | tail -1)
    if [ -z "$got" ]; then
        echo "[WARN] no SESSION line: the start died before the model ran - read the digest, fix, repeat the start" | tee -a "$base.log"
    elif [ "$got" != "$want" ]; then
        echo "[WARN] model ran as '$got', requested '$want'" | tee -a "$base.log"
    fi
fi
echo "=== exit $code; final message: $base.last.md; result file: $dir/$name.result.md $( [ -f "$dir/$name.result.md" ] && echo present || echo MISSING )" | tee -a "$base.log"
exit "$code"
