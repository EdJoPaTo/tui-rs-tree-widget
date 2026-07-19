# Generates media/screenshot.png for the README.
#
# VHS (https://github.com/charmbracelet/vhs) is required. It bundles ttyd and
# needs ffmpeg on the PATH. Install it with: brew install vhs
#
# Build the example first, then record. The build step makes the launch fast,
# so the compile output stays out of the capture:
#
#   cargo build --example example
#   vhs media/screenshot.tape
#
# The example opens two levels of nested nodes at launch, so the indent guides
# show on the first frame. The tape captures that frame with no keypresses. This
# keeps the performance overlay out of the image, because the example draws the
# overlay only after a keypress.
#
# VHS needs an Output line. The gif is only a byproduct (git-ignored). The
# committed asset is the PNG from the Screenshot command below.

Output media/screenshot.gif

Set Width 760
Set Height 430
Set FontSize 20
Set Padding 20

# The shell prompt needs time to start. If the tape types too early, the command
# runs late. So wait first. Then launch the example (hidden) and let the TUI
# paint before the tape shows it.
Sleep 3s
Hide
Type "cargo run --example example"
Enter
Sleep 5s
Show

# The tree launches with Bravo and Delta open, so the vertical, branch, and
# last-branch guides show immediately. Capture that frame.
Sleep 500ms
Screenshot media/screenshot.png

# Quit the example. This restores the terminal.
Type "q"
Sleep 500ms
