# Generates media/screenshot.png for the README.
#
# Requires VHS (https://github.com/charmbracelet/vhs), which bundles ttyd and
# needs ffmpeg on PATH:  brew install vhs
#
# Regenerate from the repository root:
#   vhs media/screenshot.tape
#
# It launches the interactive example, expands two levels of nested nodes so the
# indent guides render, captures a single PNG frame, and quits the example.
#
# VHS requires an Output; the resulting gif is only a byproduct (git-ignored),
# the committed asset is the PNG written by the Screenshot command below.

Output media/screenshot.gif

Set Width 760
Set Height 430
Set FontSize 20
Set Padding 20

# Build and launch the example without showing the compile output or prompt.
Hide
Type "cargo run --example example"
Enter
Sleep 5s
Show
Sleep 500ms

# Select Bravo and expand it, then descend into Delta and expand it, so the
# tree shows the vertical, branch and last-branch guides across two depths.
Down          Sleep 300ms
Down          Sleep 300ms
Right         Sleep 500ms
Down          Sleep 300ms
Down          Sleep 300ms
Right         Sleep 600ms
Down          Sleep 800ms

Screenshot media/screenshot.png

# Quit the example cleanly (restores the terminal).
Type "q"
Sleep 500ms
