# Generates media/screenshot.png for the README.
#
# Requires VHS (https://github.com/charmbracelet/vhs), which bundles ttyd and
# needs ffmpeg on PATH:  brew install vhs
#
# Build the example first, then record (running the prebuilt binary keeps the
# capture free of compile output and timing flakiness):
#
#   cargo build --example example
#   vhs media/screenshot.tape
#
# The example opens two levels of nested nodes on launch, so the indent guides
# render on the first frame. It captures that frame with no keypresses, which
# keeps the example's performance overlay (only drawn after input) out of shot.
#
# VHS requires an Output; the resulting gif is only a byproduct (git-ignored),
# the committed asset is the PNG written by the Screenshot command below.

Output media/screenshot.gif

Set Width 760
Set Height 430
Set FontSize 20
Set Padding 20

# Let the shell prompt finish initializing before typing, otherwise the command
# can be typed before the prompt is ready and run late. Then launch the example
# (hidden) and let the TUI paint before revealing it.
Sleep 3s
Hide
Type "cargo run --example example"
Enter
Sleep 5s
Show

# The tree launches with Bravo and Delta already open, so the vertical, branch
# and last-branch guides are on screen immediately. Capture that frame.
Sleep 500ms
Screenshot media/screenshot.png

# Quit the example cleanly (restores the terminal).
Type "q"
Sleep 500ms
