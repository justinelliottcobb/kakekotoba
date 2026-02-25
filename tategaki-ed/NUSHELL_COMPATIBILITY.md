# Nushell Compatibility Issue

## Problem

The tategaki-ed terminal editor **does not render properly when run from nushell**.

## Symptoms

When running the editor from nushell, you may experience:
- Blank terminal screen with only cursor visible
- No text rendering
- Incorrect cursor positioning
- Editor appears to start but nothing displays

## Root Cause

Notcurses (the terminal rendering library used by tategaki-ed) has compatibility issues with nushell's terminal handling. The exact technical reason is unclear, but it appears to be related to how nushell manages terminal state and escape sequences.

## Workaround

**Use a standard shell instead of nushell:**

```bash
# Switch to bash
bash

# Then run the editor
BINDGEN_EXTRA_CLANG_ARGS="-I/usr/lib/gcc/x86_64-linux-gnu/14/include" \
  cargo run --no-default-features --features notcurses --bin tategaki-ed-terminal test_vertical.txt
```

## Tested Working Shells

✅ **bash** - Fully working
✅ **zsh** - Fully working
✅ **sh** - Should work (POSIX shell)

❌ **nushell** - Does NOT work

## Technical Details

- The issue is specifically with notcurses rendering, not the editor logic
- The editor initializes successfully (notcurses context is created)
- Keyboard input is captured correctly
- The rendering pipeline executes without errors
- However, the rendered output does not appear on screen in nushell

## Potential Solutions (Future Investigation)

1. **Test with different notcurses options** - Try different initialization flags
2. **Alternative backends** - Consider implementing a crossterm backend as fallback
3. **Nushell team coordination** - Report issue to nushell project
4. **Environment variables** - Test if specific env vars affect rendering

## Workaround for Nushell Users

If you normally use nushell, you can:

1. Create a bash script wrapper:
```bash
#!/bin/bash
# tategaki.sh
export BINDGEN_EXTRA_CLANG_ARGS="-I/usr/lib/gcc/x86_64-linux-gnu/14/include"
cargo run --no-default-features --features notcurses --bin tategaki-ed-terminal "$@"
```

2. Run from bash temporarily:
```nushell
# From nushell
bash -c "BINDGEN_EXTRA_CLANG_ARGS='-I/usr/lib/gcc/x86_64-linux-gnu/14/include' cargo run --no-default-features --features notcurses --bin tategaki-ed-terminal test_vertical.txt"
```

3. Use `bash` command to enter bash, run editor, then exit back to nushell

## Related Issues

This issue may be related to:
- Nushell's external command handling
- Terminal mode/state management differences
- ANSI escape sequence processing
- Raw mode terminal handling

## Detection

You can detect if you're running in nushell:
```bash
echo $SHELL
# or
ps -p $$
```

If you see `nu` or `nushell`, you're in nushell and may experience rendering issues.

## Status

- **Discovered:** 2025-11-30
- **Status:** Known issue, no fix planned
- **Recommended action:** Use bash/zsh instead
- **Impact:** High for nushell users, zero for other shell users

---

**Note:** This is a limitation of the current notcurses-based implementation. Future versions may include alternative rendering backends that work with nushell.
