Output demo.gif

Require echo
Require vega

Set Shell "bash"
Set FontSize 16
Set Width 1200
Set Height 600

Sleep 1s

Type "vega --version" Sleep 500ms Enter
Sleep 1s

Type "vega help" Sleep 500ms Enter
Sleep 3s Ctrl+l Sleep 500ms

Type "vega pull all" Sleep 500ms Enter

# Select english language (default)
Sleep 2s Enter Sleep 500ms

# Select download dir (default)
Sleep 2s Enter Sleep 500ms

# Do not download images
Sleep 2s Type "N" Sleep 500ms Enter

# Wait for download to finish
Sleep 15s
