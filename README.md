# mmpd

mmpd turns a MIDI keyboard or controller hooked up to your computer into a versatile macro pad. The aim is to assign
behavior to keys and controllers, while differentiating based on the application that is currently focused.

Essentially, think of it as an additional keyboard that does custom things based on what application you're working with.

You can of course also set up actions that work regardless of the application.

Initially written for use on Linux distributions using the X windowing system, it is structured with
the intent to allow implementing it for other platforms, though this has not been done yet.

## Current status: tentatively ready for some use

What's implemented so far:

- Detecting focused window (window class, name)
- Connecting to a MIDI input device, receiving and parsing its messages
- Data structures for describing:
  - Scopes (focused window matching)
    - With flexible string matching
  - Actions (to be run in response to MIDI events)
  - Event matchers (describes an event to matched to trigger an event)
    - Midi Event matcher with flexible parameter value matching options
  - Macros (combining scopes, event matchers, and actions into one package)
- Configuration: YAML parser to intermediary "RawConfig" format, plus a parser
  from RawConfig into the aforementioned data structures
- Command line interfaces covering
  - Picking a config file or loading one from default location
  - list-midi-devices subcommand
  - monitor subcommand (to view incoming events without running macros)
  - (no subcommand) listening for events and running configured macros in response

There's documentation on the configuration format in [docs/config.md](https://github.com/michd/midi-macro-pad/blob/main/docs/config.md)
including some future plans.

## To do:

- Fix up focused window checking, so it doesn't need to use `Command`, use
  a library instead.
- Implement the state keeping component (MIDI etc) and precondition data structures
- Add some action enum types that allow control of the program (like exiting it, reloading config)
- Investigate portability to non-linux platforms

## Dependencies

- [xdotool](https://www.semicomplete.com/projects/xdotool/) (get it through your system's package manager)
  
  xdotool is used both through its library (libxdo) and directly via the shell. This may be
  improved on later by using X libraries directly.

