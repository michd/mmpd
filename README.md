# MIDI Macro Pad

MIDI Macro Pad turns the MIDI Keyboard or controller you have hooked up to your computer into a versatile
Macro pad. The aim is to assign behavior to keys and controls, differentiating based on the application that is
currently focused.

Initially written for use on Linux distributions using the X windowing system, it is structured with
the intent to allow implementing it for other platforms.

## Current status: not ready for use

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
  
- Limited unit test coverage for the data structures

The program's "listen" command currently loads the "testcfg.yml" config file,
loads it into the data structures and uses it, which works.

There's documentation on the configuration format in [docs/config.md](https://github.com/michd/midi-macro-pad/blob/main/docs/config.md)
including some future plans.

## To do:

- Extensively cover configuration parsing in unit tests
- Build out command line interface (defaulting to config file etc)
- Allow specifying midi input name pattern in config file
- Add a "monitor" verb to monitor incoming MIDI messages for debugging
- Fix up focused window checking so it doesn't need to use `Command`, use
  a library instead.
- Implement the state keeping component (MIDI etc) and precondition data structures
- Add some action enum types that allow control of the program (like exiting it, reloading config)
- Investigate portability to non-linux platforms

## Dependencies

- [xdotool](https://www.semicomplete.com/projects/xdotool/) (get it through your system's package manager)
  
  xdotool is used both through its library (libxdo) as well as directly via the shell. This may be
  improved on later by using X libraries directly.

