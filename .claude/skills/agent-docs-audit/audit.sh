#!/usr/bin/env bash
# Audits the agent instruction set of this repository. Read-only: prints findings, changes nothing.
# Written for bash 3.2 (macOS system bash): no associative arrays, no `**` globbing.
cd "$(git rev-parse --show-toplevel 2>/dev/null || echo .)" || exit 1

fail=0
note(){ echo "  [$1] $2"; [ "$1" = FAIL ] && fail=1; return 0; }

# Frontmatter helpers: the lines between a first-line `---` and the next `---`.
fm(){ awk 'NR == 1 { if ($0 != "---") exit; next } /^---$/ { exit } { print }' "$1"; }
fm_field(){ fm "$1" | sed -n "s/^$2:[[:space:]]*//p" | head -1 \
            | sed -e 's/[[:space:]]*$//' -e 's/^"\(.*\)"$/\1/' -e "s/^'\(.*\)'\$/\1/"; }
body(){ awk 'NR == 1 && $0 == "---" { f = 1; next } f == 1 && /^---$/ { f = 2; next } f != 1 { print }' "$1"; }
# A glob matches if git tracks at least one file under it.
glob_hits(){ [ -n "$(git ls-files -- ":(glob)$1" | head -1)" ]; }

DOCS="AGENTS.md CLAUDE.md GEMINI.md website/AGENTS.md"
for f in .claude/rules/*.md .claude/skills/*/SKILL.md .cursor/rules/*.mdc .ai/*.md; do
  [ -f "$f" ] && DOCS="$DOCS $f"
done

echo "== 1. Startup context budget =="
for f in AGENTS.md CLAUDE.md GEMINI.md; do
  [ -f "$f" ] || { note FAIL "$f is missing"; continue; }
  n=$(wc -l < "$f" | tr -d ' ')
  if [ "$n" -le 200 ]; then note OK "$f = $n lines"; else note WARN "$f = $n lines, over 200"; fi
done
tot=$(cat AGENTS.md CLAUDE.md | wc -c | tr -d ' ')
note INFO "Claude Code eager load: AGENTS.md + CLAUDE.md = ${tot}b (~$((tot / 4)) tokens)"

echo "== 2. Imports =="
if [ "$(head -1 CLAUDE.md)" = "@AGENTS.md" ]; then note OK "CLAUDE.md line 1 imports AGENTS.md"
else note FAIL "CLAUDE.md line 1 is not @AGENTS.md - Claude Code will not load the contract"; fi
hits=$(grep -nHE '(^|[[:space:]])@[A-Za-z0-9._/~-]+\.md' $DOCS 2>/dev/null | grep -v '`@' | grep -v '^CLAUDE.md:1:@AGENTS.md$')
if [ -n "$hits" ]; then echo "$hits" | sed 's/^/  [WARN] eager import: /'; else note OK "no other eager imports"; fi

echo "== 3. Skills: frontmatter and .agents/skills symlinks =="
for s in .claude/skills/*/SKILL.md; do
  [ -f "$s" ] || continue
  dir=$(basename "$(dirname "$s")")
  name=$(fm_field "$s" name); desc=$(fm_field "$s" description)
  [ "$name" = "$dir" ] || note FAIL "$s: name '$name' differs from directory '$dir'"
  case "$desc" in
    "Use when"*|"Use before"*) ;;
    "") note FAIL "$s: description missing" ;;
    *) note WARN "$s: description does not start with 'Use when'" ;;
  esac
  link=.agents/skills/$dir
  if [ -L "$link" ] && [ "$(cd "$link" 2>/dev/null && pwd -P)" = "$(cd "$(dirname "$s")" && pwd -P)" ]; then
    note OK "$dir: frontmatter valid, $link resolves"
  else note WARN "$dir: $link is missing or does not point to .claude/skills/$dir"; fi
done
for l in .agents/skills/*; do
  [ -L "$l" ] && [ ! -e "$l" ] && note FAIL "$l is a dangling symlink"
done

echo "== 4. Path-scoped rules and Cursor parity =="
for r in .claude/rules/*.md; do
  [ -f "$r" ] || continue
  n=$(basename "$r" .md); m=.cursor/rules/$n.mdc
  cg=$(fm "$r" | grep -oE '"[^"]+"' | tr -d '"' | LC_ALL=C sort)
  [ -n "$cg" ] || note WARN "$r: no paths: globs in frontmatter - the rule loads nowhere"
  set -f
  for g in $cg; do glob_hits "$g" || note WARN "$n: glob '$g' matches no tracked file"; done
  set +f
  if [ ! -f "$m" ]; then note WARN "$r has no Cursor pointer $m"; continue; fi
  mg=$(fm_field "$m" globs | tr ',' '\n' | sed 's/^[[:space:]]*//;s/[[:space:]]*$//' | grep -v '^$' | LC_ALL=C sort)
  if [ "$cg" != "$mg" ]; then
    note WARN "$n: globs differ - .claude [$(printf '%s' "$cg" | tr '\n' ' ')] vs .cursor [$(printf '%s' "$mg" | tr '\n' ' ')]"
  elif ! body "$m" | grep -qF "$r"; then note WARN "$m does not point to $r"
  else note OK "$n: $m mirrors $r"; fi
done
for m in .cursor/rules/*.mdc; do
  aa=$(fm_field "$m" alwaysApply)
  case "$aa" in true|false) ;; *) note WARN "$m: alwaysApply is '$aa'";; esac
done

echo "== 5. Paths named in the docs exist =="
miss=0
for f in $DOCS; do
  for p in $(grep -oE '`[.A-Za-z0-9_/-]+/[.A-Za-z0-9_-]*`' "$f" | tr -d '`' | sort -u); do
    # Not repository paths: context7 IDs (/org/lib), branch and ref names, GitHub
    # actions, placeholders (X.Y.Z, NEW, X.md) and hypothetical future filenames.
    case "$p" in /*|*X.Y.Z*|http*|*/|refs/*|origin/*|release/*|feat/*|fix/*|rustsec/*) continue;; esac
    case "$p" in docs/superpowers/*|*/NEW.*|*/X.*|*_v[0-9]*.png) continue;; esac
    # Site docs name paths relative to website/ and its content directory.
    [ -e "$p" ] || [ -e "$(dirname "$f")/$p" ] || [ -e "website/$p" ] || [ -e "website/src/content/docs/$p" ] \
      || { echo "  [WARN] $f names missing path: $p"; miss=1; }
  done
done
[ "$miss" -eq 0 ] && note OK "every backticked path resolves"

echo "== 6. One verification gate =="
bad=$(grep -nHE 'cargo clippy --all-targets' $DOCS CONTRIBUTING.md .github/PULL_REQUEST_TEMPLATE.md 2>/dev/null | grep -v -- '--locked')
if [ -n "$bad" ]; then echo "$bad" | sed 's/^/  [WARN] clippy without --locked: /'; else note OK "every clippy gate uses --locked"; fi
# Any line naming both npm test and npm run build is a statement of the site gate.
bad=$(grep -nHE 'npm test.*npm run build' $DOCS CONTRIBUTING.md .github/PULL_REQUEST_TEMPLATE.md 2>/dev/null | grep -v 'npm ci')
if [ -n "$bad" ]; then echo "$bad" | sed 's/^/  [WARN] website gate without npm ci: /'; else note OK "every website gate uses npm ci"; fi

echo "== 7. Tool-specific text stays out of AGENTS.md =="
hits=$(grep -nE 'view_file|grep_search|find_by_name|AskUserQuestion|TodoWrite|superpowers:' AGENTS.md)
if [ -n "$hits" ]; then echo "$hits" | sed 's/^/  [WARN] AGENTS.md: /'; else note OK "AGENTS.md names no harness-specific tools"; fi

echo "== 8. Attribution guards =="
if [ "$(git config core.hooksPath)" = ".githooks" ]; then note OK "core.hooksPath = .githooks"
else note WARN "hooks not enabled in this clone: git config core.hooksPath .githooks"; fi
for h in .githooks/commit-msg .githooks/pre-push; do
  [ -x "$h" ] || note FAIL "$h missing or not executable"
done
if grep -q '"commit": ""' .claude/settings.json 2>/dev/null && grep -q '"pr": ""' .claude/settings.json; then
  note OK ".claude/settings.json disables commit and PR attribution"
else note FAIL ".claude/settings.json does not disable both commit and PR attribution"; fi
# Same patterns as the commit-msg hook, applied to each message body line.
trail=$(git log -50 --format='%B' | grep -i -E -f .githooks/attribution.ere)
if [ -n "$trail" ]; then echo "$trail" | sed 's/^/  [WARN] attribution in recent history: /'; else note OK "last 50 commits carry no attribution"; fi

echo "== 9. Volatile facts =="
note INFO "context7 IDs in AGENTS.md are dated; re-resolve one if a query returns nothing"
grep -hoE 'verified [0-9]{4}-[0-9]{2}-[0-9]{2}' AGENTS.md CLAUDE.md GEMINI.md 2>/dev/null | sort -u | sed 's/^/  [INFO] /'

echo; if [ "$fail" -eq 0 ]; then echo "No blocking problems."; else echo "Blocking problems found."; fi
exit "$fail"
