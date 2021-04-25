#!/usr/bin/osascript
-- With thanks to this StackOverflow answer by Albert
-- https://stackoverflow.com/a/5293758/1019228

global frontApp, frontAppName, frontAppClass, windowTitle
set windowTitle to ""

tell application "System Events"
    -- Get the focused application
    set frontApp to first application process whose frontmost is true
    -- Grab name and display name of focused application, these will
    -- be the values we populate window_class with
    set frontAppName to name of frontApp
    set frontAppClass to displayed name of frontApp

    -- Get the window title
    tell process frontAppName
        tell (1st window whose value of attribute "AXMain" is true)
            set windowTitle to value of attribute "AXTitle"
        end tell
    end tell

end tell

-- Print app class, app name, and window title to STDOUT on their own lines
copy frontAppClass & "\n" & frontAppName & "\n" & windowTitle to stdout