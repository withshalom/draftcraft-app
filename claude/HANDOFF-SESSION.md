# Session handoff: DraftCraft DWG fluency

Written 2026-11-11 in the Claude Code session "DraftCraft DWG work setup" (`session_01F67RspwBCoRVMtoi14joCg`),
before the model is switched. The user is Shalom. Nothing has been changed, committed or pushed yet.

Shalom's own handoff, `HANDOFF-DWG-FLUENCY.md`, was uploaded to this session
(`/root/.claude/uploads/5e366a1d-6523-5837-9094-c623e2ad21e6/4677f8fc-HANDOFF-DWG-FLUENCY.md`). Read it first:
it has the goal, priorities, Discord state and the eight decisions for Shalom. This file adds what this
session checked and agreed. `DWG-READER-ROADMAP.md` and `WRITER-LOSSES.md` live in the claude.ai Project
"DraftCraft", which this session can't reach; they were not uploaded and have not been read.

## How Shalom wants to work

- Experimental work goes in the fork, `withshalom/draftcraft-app`, on branches, with a draft PR into the
  fork's `main`. Never push to `main` directly: every push to the fork's `main` deploys app.draftcraft.dev.
- Finished work goes upstream as a PR from a fork branch to `SkryxDev/DraftCraft` `main`, as PR #7 did.
  Shalom opens those PRs for now (see Access).
- Don't merge experiments into the fork's `main` early. Keep them unmerged, or behind a default that
  doesn't change behaviour (the DWG reader stays `acadrust` unless chosen).
- Follow `AGENTS.md` in the repo: never crash, clean-room, everything is a command, `cargo xtask ci`
  before every commit.

## Access, checked

| Repository | Access | Local path |
|---|---|---|
| `withshalom/draftcraft-app` | push + GitHub tools (dry-run push accepted) | `/home/user/draftcraft-app`, branch `claude/ecstatic-lovelace-2qhtjw` at `3d1cf5e`; remote `upstream` = SkryxDev/DraftCraft added and fetched (depth 200) |
| `withshalom/blackcad` | push + GitHub tools (dry-run push accepted) | `/home/user/blackcad`, `main` at `e9d80db` |
| `SkryxDev/DraftCraft` | git read only, no API | via the `upstream` remote |
| `withshalom/draftcraft-dev` | git read only | not cloned |
| `withshalom/cadcraft` | git read only | not cloned |

- **Open question, Shalom's to answer separately:** request push access to `SkryxDev/DraftCraft`. It is
  the only way to see upstream PR discussions and CI. Don't ask again unprompted and don't request it.
- Read-only clones: `GIT_LFS_SKIP_SMUDGE=1 git clone --depth 1 https://github.com/<owner>/<repo> /home/user/<owner>/<repo>`.

## Build environment, checked

- Rust 1.97.0 (workspace needs ≥ 1.95), 4 cores, ~30 GB free disk.
- **wasm32-unknown-unknown is not installed**, so `cargo xtask wasm` (part of `cargo xtask ci`) fails until
  `rustup target add wasm32-unknown-unknown`. static.rust-lang.org is reachable. Not done yet.
- Headless machine: tests, CLI and `draftcraft-cli ... --export` work; GUI screenshots over the control
  channel may need a virtual display (not tried).
- The container is temporary: commit and push anything worth keeping.

## Facts verified against the repositories

Matches Shalom's handoff:
- Fork `main` `3d1cf5e` (2026-11-09), version 2.0.2. Fork PRs #1 and #2 both merged; none open.
- Upstream `main` `24e7d4f` (2026-11-10), version 2.1.0. PR #7 merged as `22ffdfa` on 2026-11-10.
- Fork is 16 ahead / 13 behind upstream; merge base `5ac7640`. The DWG commits exist on both sides under
  different hashes.
- `crates/dwg`, `crates/io/src/dwg_read.rs`, `docs/dwg.md` are identical on fork and upstream.
- Sizes on upstream: `read/` 7,711 lines, `records/` 4,260, `write/` 3,220, `dwg_read.rs` 619.
- Reader chosen only by `DRAFTCRAFT_DWG_READER` (`crates/io/src/lib.rs:100`), default `acadrust`.
- AC1021 (2007–2009) container not implemented (`crates/dwg/src/read/mod.rs:5`).
- Nothing outside `crates/dwg` calls the writer; `write/` has no tests.

Disagrees with or missing from Shalom's handoff:
1. `dwg-codec-upstream` is **not** merged into the fork's `main`; it was merged upstream via PR #7. The
   fork's `main` holds the same changes under other hashes, through `draftcraft-dwg`.
2. Upstream PR #10: a `refs/pull/10/merge` ref exists (suggests open); state can't be confirmed without API.
3. The fork has 17 branches (`rebrand`, `web`, `rustdraft`, `probe-release`, `acadrust-web`, `app-logo`,
   `autostyle`, `draftcraft-archive-pr`, `fix/rebrand-io-test`, `gh-pages`, `logo-and-menu`,
   `readme-credits`, `title-bar-logo`, `DraftCraft`, plus the two DWG branches and `main`). Others not checked.

## Where PR #7 left things

The DWG codec is in upstream `main` and release 2.1.0, but opening and saving still go through acadrust.
Mapped natively: 14 entity types, layers, linetypes, text styles, blocks, layouts, main header variables.
`auto` never picks native on a real AutoCAD file because non-graphic objects (dictionaries, XRECORD,
VISUALSTYLE, …) are unmapped, so `Coverage` is never empty. `docs/dwg.md` sets the order: non-graphic
objects into `Drawing::foreign` → missing entities → 2018 writer with read-back check → acadrust behind a
feature for 2007 files and older save-as versions.

## Next steps (proposed, waiting for Shalom's go-ahead)

1. **Sync the fork with upstream 2.1.0.** Install the wasm target; branch from the fork's `main`, merge
   `upstream/main`, resolve conflicts (expected only in branding/web files, not DWG), run `cargo xtask ci`,
   push, open a draft PR into the fork's `main`, subscribe to it. Merging it deploys 2.1.0 to the web.
2. **Roadmap Phase 0:** DWG reader setting in Preferences (Acadrust, Native, Auto); a notice on open
   listing what was left out; `draftcraft-cli dwg-compare` diffing the two readers on a file. New strings
   need `it.tsv` (and `zh.tsv` for menu entries); a new command needs a ribbon place and an icon.
3. **Phase 1 (`Drawing::foreign`)** only after Shalom settles handoff decisions 1 (follow-ups to PR #7 as
   separate upstream PRs?) and 2 (how foreign objects are held; needs SkryxDev's agreement).

## Notes

- Use they/them for people in the Discord and upstream unless their pronouns are stated.
- Don't put model names in commits, PRs or code.
- PR description footer and commit trailers come from the session's attribution reminder.
