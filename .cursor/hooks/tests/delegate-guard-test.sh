#!/bin/bash
# Cases for delegate-guard.sh as "expected|command", run with CURSOR_DELEGATE=1.
# Run: bash .cursor/hooks/tests/delegate-guard-test.sh .cursor/hooks/delegate-guard.sh
hook=$1
cases=(
  # --- reads a review needs
  'ALLOW|git status --short'
  'ALLOW|git log --oneline -n 20'
  'ALLOW|git diff .github/workflows/release.yml'
  'ALLOW|git show HEAD:AGENTS.md'
  'ALLOW|git branch --show-current'
  'ALLOW|git branch -a'
  'ALLOW|git branch --contains HEAD'
  'ALLOW|git stash list'
  'ALLOW|git config --get core.hooksPath'
  'ALLOW|git -C ../other log -n 3'
  'ALLOW|gh pr view 2 -R owner/repo'
  'ALLOW|gh run list -L 5'
  'ALLOW|gh pr diff 2'
  'ALLOW|cargo clippy --all-targets --locked -- -D warnings'
  'ALLOW|cargo test --locked'
  'ALLOW|cargo publish --dry-run --locked'
  'ALLOW|npm ci'
  'ALLOW|rm notes.tmp'
  'ALLOW|rg -n "git push" AGENTS.md'
  'ALLOW|grep -rn "gh api" .github'
  'ALLOW|echo "rm -rf target"'
  $'ALLOW|cat > n.md <<\'EOF\'\ngit push origin main\nrm -rf /\nEOF'
  'ALLOW|bash .claude/skills/agent-docs-audit/audit.sh'
  # --- git mutations in every spelling
  'DENY|git commit -m x'
  'DENY|git "push"'
  'DENY|/usr/bin/git commit -m x'
  'DENY|git --git-dir /tmp/r commit -m x'
  'DENY|git --work-tree /tmp commit -m x'
  'DENY|git -C . push'
  'DENY|git commit-tree HEAD^{tree} -m x'
  'DENY|git update-ref refs/heads/main HEAD'
  'DENY|git fetch origin'
  'DENY|git remote add x y'
  'DENY|git branch new-branch'
  'DENY|git branch -D x; git branch --show-current'
  'DENY|git stash drop; git stash list'
  'DENY|git stash'
  'DENY|git config user.name x; git config --get user.name'
  'DENY|env GIT_DIR=.git git commit -m x'
  'DENY|bash -c "git push"'
  'DENY|echo $(git commit -m x)'
  'DENY|echo "git push" | sh'
  'DENY|eval "git reset --hard"'
  'DENY|true && git tag v1'
  # --- GitHub, publishing, rm
  'DENY|gh pr create -t x'
  'DENY|gh --repo o/r pr create -t x'
  'DENY|gh -R o/r pr merge 2'
  'DENY|gh api repos/o/r'
  'DENY|gh "api" repos/o/r'
  'DENY|gh secret set X'
  'DENY|gh workflow run release.yml'
  'DENY|cargo publish --locked'
  'DENY|cargo install grove-git'
  'DENY|npm publish'
  'DENY|rm -rf target'
  'DENY|rm "-rf" target'
  'DENY|/bin/rm -r target'
  'DENY|rm --recursive target'
  # --- second review: bypasses
  'DENY|git -c alias.p=push p'
  "DENY|git -c alias.p='!git push' p"
  'DENY|git config --file list --add user.name attacker'
  'DENY|git config set user.name get'
  'DENY|echo git push | sh'
  "DENY|printf '%s %s\n' git push | sh"
  $'DENY|sh <<\'EOF\'\ngit push\nEOF'
  'DENY|cargo --locked publish'
  'DENY|cargo +nightly publish'
  'DENY|npm --prefix /tmp publish'
  'DENY|gh pr -R o/r create -t x'
  'DENY|gh repo clone o/r /tmp/clone'
  'DENY|sudo -u root git push'
  'DENY|env -u GIT_DIR git push'
  'DENY|git hash-object -w blob.txt'
  'DENY|git send-pack . HEAD'
  'DENY|git fetch-pack . HEAD'
  'DENY|git receive-pack .'
  'DENY|git reflog delete HEAD@{1}'
  'DENY|git pack-refs --all'
  'DENY|git commit-graph write'
  'DENY|git remote update'
  'DENY|git tag -d v1'
  'DENY|git tag -a v1 -m x'
  'DENY|git worktree add /tmp/w'
  # --- security review: programs git runs, find -exec, heredocs into a shell
  "DENY|git -c core.pager='sh -c \"git push\"' log"
  'DENY|git -c core.sshCommand=evil fetch'
  'DENY|git -c diff.x.textconv=evil diff'
  'DENY|git -c credential.helper=evil status'
  'DENY|git --config-env=core.pager=X log'
  'DENY|GIT_PAGER=evil git log'
  'DENY|env GIT_SSH_COMMAND=evil git status'
  'DENY|GIT_EXTERNAL_DIFF=evil git diff'
  'DENY|find . -name x -exec git push {} ;'
  'DENY|find . -execdir rm -rf {} +'
  $'DENY|cat <<\'EOF\' | sh\ngit push\nEOF'
  'ALLOW|git -c color.ui=never log -n 3'
  'ALLOW|git -c core.quotepath=off status'
  'ALLOW|find . -name "*.md" -exec grep -l Grok {} +'
  # --- security review, round 4
  'DENY|export GIT_PAGER=evil; git log'
  'DENY|declare -x GIT_SSH_COMMAND=evil'
  'DENY|GIT_DIR=/tmp/r git log'
  'DENY|HOME=/tmp/h git log'
  'DENY|GIT_CONFIG_GLOBAL=/tmp/c git log'
  'DENY|git -c difftool.x.cmd=evil difftool'
  'DENY|git difftool -x evil'
  'DENY|git mergetool'
  'DENY|git instaweb'
  'DENY|git grep -O foo'
  'DENY|git grep --open-files-in-pager=evil foo'
  'DENY|find . -name "*.tmp" -delete'
  'ALLOW|export LC_ALL=C; git log -n 1'
  'ALLOW|git grep -n foo'
  'ALLOW|find . -name "*.md"'
  # --- second review: reads that must pass
  'ALLOW|git remote -v'
  'ALLOW|git remote get-url origin'
  'ALLOW|git tag -l'
  'ALLOW|git worktree list'
  'ALLOW|git config user.name'
  'ALLOW|git config core.hooksPath'
  'ALLOW|git config --show-scope user.name'
  'ALLOW|git config get user.name'
  'ALLOW|git hash-object README.md'
  'ALLOW|git reflog -n 5'
  'ALLOW|git commit-graph verify'
  'ALLOW|grep -rn "gh api" .github; bash .claude/skills/agent-docs-audit/audit.sh'
)
pass=0; fail=0
for c in "${cases[@]}"; do
  want=${c%%|*}; cmd=${c#*|}
  got=$(jq -n --arg c "$cmd" '{command:$c, hook_event_name:"beforeShellExecution"}' \
        | CURSOR_DELEGATE=1 "$hook" 2>/dev/null | jq -r '.permission' 2>/dev/null | tr '[:lower:]' '[:upper:]')
  if [ "$got" = "$want" ]; then pass=$((pass+1)); else fail=$((fail+1)); printf 'MISMATCH want=%s got=%s :: %q\n' "$want" "$got" "$cmd"; fi
done
# Malformed payloads deny.
for payload in 'not json' '{"command": 5}' '{}'; do
  got=$(printf '%s' "$payload" | CURSOR_DELEGATE=1 "$hook" 2>/dev/null | jq -r '.permission' 2>/dev/null)
  if [ "$got" = deny ]; then pass=$((pass+1)); else fail=$((fail+1)); printf 'MISMATCH want=deny got=%s :: payload %s\n' "$got" "$payload"; fi
done
# Outside a delegated run everything passes.
got=$(jq -n '{command:"git push"}' | env -u CURSOR_DELEGATE "$hook" | jq -r '.permission')
if [ "$got" = allow ]; then pass=$((pass+1)); else fail=$((fail+1)); echo "MISMATCH: guard active without CURSOR_DELEGATE"; fi
echo "pass=$pass fail=$fail"
