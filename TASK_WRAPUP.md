# Task Wrap-Up Checklist

This document outlines the process for properly completing a task, ensuring all relevant documentation is updated and changes are properly tracked.

## When to Use This Document

Use this checklist whenever:
- Completing a feature implementation
- Fixing a bug
- Making any non-trivial code change
- Abandoning or deferring work on something

## Pre-Wrap-Up Checklist

### 1. Code Quality

- [ ] **Run tests:** `cargo test` - ensure all tests pass
- [ ] **Build successfully:** `cargo build --release` - no compilation errors
- [ ] **Run clippy:** `cargo clippy` - address any warnings (or document intentional ones)
- [ ] **Install and test:** `install -Dm755 target/release/cleanshitx ~/.local/bin/openshotx` then test the feature

### 2. Documentation Updates

Update these files as appropriate:

#### PROGRESS.md (for completed work)
Add new entries under "completed" with:
- Date
- Feature/bug name
- List of specific changes made
- Any relevant metrics (performance improvements, etc.)

#### PROGRESS.md (for abandoned/deferred work)
Add under the relevant section (in progress, todo, blockers, future):
- What was attempted
- Why it was abandoned/deferred
- Known issues or limitations

#### README.md (for user-facing changes)
- Update "what actually works right now" if new features were added
- Update options section if new CLI flags were added
- Update technicals section if implementation approach changed
- Add/remove keyboard shortcuts if bindings were changed

#### ROADMAP.md (for structural changes)
- Mark items as done if milestone was reached
- Update priority ordering if focus shifted
- Document technical challenges if new ones were discovered

#### SCROLLING_CAPTURE.md (or similar feature docs)
For complex features, maintain separate documentation:
- Current state of implementation
- Known issues and limitations
- Attempted solutions and results
- Recommendations for next steps

### 3. Git Status Review

```bash
git status
git diff --stat
```

- [ ] Review all uncommitted changes
- [ ] Ensure no sensitive data (credentials, tokens) is included
- [ ] Confirm all modified files are intentional

### 4. Commit Strategy

**For completed features:**
- Commit with a clear message describing what was done
- Example: `feat: add clipboard integration for screenshot captures`

**For work-in-progress that shouldn't be committed:**
- Document in PROGRESS.md under appropriate section
- Changes remain uncommitted but are tracked in documentation

**For abandoned work:**
- Document why it was abandoned in PROGRESS.md
- If there are code changes, keep them uncommitted or revert

## Feature Documentation Template

When adding a new feature to documentation, include:

```markdown
### vX.X.X - Feature Name (YYYY-MM-DD)
- [x] Brief description of change 1
- [x] Brief description of change 2
- [ ] Incomplete item (if any)

**Technical details:**
- Implementation approach used
- Why this approach was chosen
- Any trade-offs made

**Testing:**
- How this was tested
- Any known limitations
```

## Known Issues Documentation Template

When documenting a problem:

```markdown
### Issue: Brief Description

**Problem:** What is broken or suboptimal

**Root Cause:** Why this happens

**Impact:** Who this affects and how

**Potential Solutions:**
1. Solution A - pros/cons
2. Solution B - pros/cons

**Recommendation:** What should be done (or "leave as-is" if working as intended)
```

## Example: Wrap-Up for v0.2.2

```
## v0.2.2 - clipboard integration & keybindings (2026-04-26)

### Completed
- [x] add clipboard integration for ALL capture types
  - copy_image_to_clipboard() function in capture module
  - uses wl-copy (Wayland) or xclip (X11) with image/png MIME type
- [x] add Hyprland keybindings for quick access
- [x] install binary to ~/.local/bin/openshotx

### Testing
- Captured area with Super+Ctrl+1 → image copied to clipboard ✓
- Captured screen with Super+Ctrl+4 → image copied to clipboard ✓

### Notes
- X11 requires xclip package (added to install instructions)
- Wayland portal dialog behavior is a security feature, not a bug
- X11 GTK overlay goes straight to region selection (no dialog)
```

## Git Workflow Summary

| Scenario | Action |
|----------|--------|
| Feature complete, tested, documented | `git add` + `git commit` |
| Work-in-progress, want to save | Keep uncommitted, document in PROGRESS.md |
| Abandoned work | Revert or keep uncommitted, document why |
| Hotfix needed | Commit with `fix:` prefix, document issue |

## Quick Reference Commands

```bash
# Check status
git status

# Review changes
git diff

# Run full quality checks
cargo test && cargo clippy && cargo build --release

# Install for testing
install -Dm755 target/release/cleanshitx ~/.local/bin/openshotx

# Update docs (example patterns)
# See PROGRESS.md, README.md, ROADMAP.md for existing formats
```

## Final Reminder

**Always test before wrapping up.** A feature isn't complete until:
1. Code compiles without errors
2. All existing tests pass
3. New functionality works as expected
4. Documentation accurately reflects the current state