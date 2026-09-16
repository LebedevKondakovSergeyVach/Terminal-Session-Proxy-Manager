# Archive

Design documents for work that has shipped. They are kept for the reasoning
they carry — why something was built the way it was, and what it replaced —
not as instructions.

**Nothing here is current.** Each document describes the state of the project
on the day it was written, and several describe problems that no longer exist.
Do not follow one as guidance, and do not update one to match the code: if a
statement here disagrees with the repository, the repository is right.

The authorities are, in order:

| For | Read |
| :--- | :--- |
| The contract you must follow | [`AGENTS.md`](../../AGENTS.md) |
| Anything under `website/` | [`website/AGENTS.md`](../../website/AGENTS.md) |
| Architecture and design constraints | [`.ai/PROJECT_OVERVIEW.md`](../PROJECT_OVERVIEW.md) |
| Branching and releases | [`.ai/GIT_WORKFLOW.md`](../GIT_WORKFLOW.md) |
| Verification | [`.ai/WORKFLOW_GUIDE.md`](../WORKFLOW_GUIDE.md) |

## Why these moved

They lived in `docs/`, which `README.md` presents as the user documentation and
which the site generator treats as its source directory. Two consequences:
a reader browsing `docs/` met 1,700 lines of internal planning, and every edit
to them triggered `website.yml` and `pages.yml` for a build whose output could
not change — the manifest never listed them.

## Contents

| Document | Shipped as |
| :--- | :--- |
| [`2026-08-22-website-rebuild-design.md`](2026-08-22-website-rebuild-design.md) | The `website/` rebuild — the generator, `base`-aware links, Git-ignored generated pages |
| [`2026-08-22-website-rebuild-plan.md`](2026-08-22-website-rebuild-plan.md) | The step-by-step execution of that design |

Both predate the 2.3.0 release. Known to be out of date in them: the asset
filenames (`banner_new.jpg` and friends were renamed), the treatment of
`starlight-theme-md3`, and the claim that generated pages are only `.md` —
most are `.mdx`, and the ignore rule now covers both.
