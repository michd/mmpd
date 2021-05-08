#!/usr/bin/osascript
-- With thanks to this StackOverflow answer by Albert
-- https://stackoverflow.com/a/5293758/1019228

global frontApp, frontAppName, frontAppClass, windowTitle, executablePath
set windowTitle to ""

tell application "System Events"
    -- Get the focused application
    set frontApp to first application process whose frontmost is true
    -- Grab name and display name of focused application, these will
    -- be the values we populate window_class with
    set frontAppName to name of frontApp
    set frontAppClass to displayed name of frontApp
    -- Get the absolute path of the application owning this window
    set executablePath to POSIX path of (application file of frontApp)

    -- Get the window title
    tell process frontAppName
        tell (1st window whose value of attribute "AXMain" is true)
            set windowTitle to value of attribute "AXTitle"
        end tell
    end tell
end tell

-- Print app class, app name, window title, executable path to STDOUT on their own lines
copy frontAppClass & "\n" & frontAppName & "\n" & windowTitle & "\n" & executablePath to stdout