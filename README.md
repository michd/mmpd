# MIDI Macro Pad

MIDI Macro Pad turns the MIDI Keyboard or controller you have hooked up to your computer into a versatile
Macro pad. The aim is to assign behavior to keys and controls, differentiating based on the application that is
currently focused.

Initially written for use on Linux distributions using the X windowing system, it is structured with
the intent to allow implementing it for other platforms.

## Current status: proof of concept

All the parts are there to prove the basic functionality:
- Detecting focused window (collecting window class and name)
- Connecting to a MIDI input device and parsing its messages
- Sending key sequences based on specific MIDI input messages and focused window

The mapping of midi messages to actions taken is hardcoded just to prove the point.
Further plans are to work out a robust, flexible, extensible configuration file format defining
midi message filters and sequences of actions to be run when those match. It will be a JSON format
with a specified structure.

## Dependencies

- [xdotool](https://www.semicomplete.com/projects/xdotool/) (get it through your system's package manager)
  
  xdotool is used both through its library (libxdo) as well as directly via the shell. This may be
  improved on later by using X libraries directly.

