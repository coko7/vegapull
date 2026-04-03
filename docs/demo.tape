Output demo.gif

Require echo
Require vegapull
Require gum
Require fx
Require jq

Set Shell "bash"
Set FontSize 16
Set Width 1200
Set Height 600

Sleep 1s

Type "vegapull --version" Sleep 500ms  Enter

Sleep 2s Ctrl+l Sleep 500ms

Type "gum spin -- vegapull packs > packs.json" Sleep 500ms Enter

Sleep 5s

Type "fx packs.json" Sleep 500ms Enter

Sleep 3s Escape Sleep 500ms

Type "pack=$(jq -r '.[0].id' packs.json) && echo $pack" Sleep 500ms Enter

Sleep 2s

Type "gum spin -- vegapull cards $pack > cards_${pack}.json" Sleep 500ms Enter

Sleep 5s Ctrl+l Sleep 500ms

Type "fx cards_${pack}.json" Sleep 500ms Enter

Sleep 3s Escape Sleep 500ms

Sleep 1s
