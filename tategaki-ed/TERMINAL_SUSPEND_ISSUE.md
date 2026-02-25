# Terminal Suspend Issue - Quick Fix

## Problem
When you suspend the terminal editor with **Ctrl+Z**, the terminal stays in raw mode and appears blank when you try to run the editor again.

## Quick Fix (Run These Commands)

```bash
# 1. Kill any suspended tategaki processes
pkill -9 tategaki-ed-terminal

# 2. Reset your terminal
reset

# 3. If reset doesn't work, try these:
stty sane
tput reset
clear
```

After running these, your terminal should work normally again.

## Why This Happens

When you press **Ctrl+Z**, the process is suspended (not terminated), so:
- The notcurses library doesn't get a chance to restore terminal state
- The terminal stays in "raw mode" (no echo, no line buffering)
- Your shell can't display properly

## Alternative: Don't Suspend, Quit Properly

Instead of **Ctrl+Z**, use the editor's quit command:
- Press `Esc` to enter Normal mode
- Type `:q` and press Enter to quit properly

This allows the editor to clean up and restore your terminal.

## For Developers: Signal Handling

To handle suspension properly, we would need to:

1. Catch SIGTSTP (Ctrl+Z signal)
2. Restore terminal before suspending
3. Catch SIGCONT (resume signal)
4. Re-initialize terminal after resume

This is complex with notcurses and may not be necessary if users know to use `:q` instead of Ctrl+Z.

## Emergency Terminal Recovery

If your terminal gets really messed up:

```bash
# Nuclear option - reset everything
reset
source ~/.bashrc  # or ~/.zshrc

# Check if any processes are stuck
ps aux | grep tategaki
kill -9 <PID>  # if any are running
```
