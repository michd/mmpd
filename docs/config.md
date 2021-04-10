# Configuration

This document details the configuration file format for mmpd.

mmpd is configured in YAML, which specifies scopes, macros, and in those macros event matchers, preconditions, and
actions to be run.

At the top level, the file looks as follows:

```yaml
version: 1

scopes:
  - ...

global_macros:
  - ...
- 
```
- `version`: Configuration file format version. Instructs the program what to expect. This is included from the
  beginning in case a future version introduced such a big overhaul that the configuration files would become
  incompatible. Including it means the program will always know what to expect, and prevents breaking changes.
- `scopes`: List of application scopes, each with its own list of macros.
- `global_macros`: List of macros, which can run regardless of which application is focused.

## Contents
- [Scopes](#scopes)
  - [String matching](#string-matching)
- [Macros](#macros)
  - [Events](#events)
    - [MIDI events](#midi-events)
    - [Value ranging](#value-ranging)
      - [MIDI](#midi)
    - [Preconditions](#preconditions)
      - [MIDI Preconditions](#midi-preconditions)
    - [Actions](#actions)
      - [key_sequence](#key_sequence)
      - [enter_text](#enter_text)
        - [Shortened version](#shortened-version)
      - [shell](#shell)
      - [wait](#wait)
        - [Shortened version](#shortened-version-1)
      - [control](#control)
        - [Control Actions](#control-actions)
      - [Variables (NOT IMPLEMENTED YET)](#variables-not-implemented-yet)
        - [Available data](#available-data)
- [Full example of a config file](#full-example-of-a-config-file)

---

## Scopes

A scope consists of a pattern to match a focused application window's title and/or window class, and a list of
associated macros that only apply if a window matching the pattern is currently focused. A scope looks like this:

```yaml
window_class:
  is: "exact text"
window_name:
  contains: "partial text"
  
macros:
  - ...
```

- `window_class`: matches against a window class of the focused window. The matching object is documented below, see
  string matching.
- `window_name`: matches against a window name of the focused window. Matching works the same.
- `macros`: List of macros that apply to this scope, documented in Macros.

### String matching

A string matching object can take several forms, all consisting of one key and a string value. The key determines how
the specified pattern gets used against the string being compared.

- `is: "pattern"`: Value must be exactly the same as the pattern string specified
- `contains: "pattern"`: Value must contain all of the pattern string.
- `start_with: "pattern"`: Value must begin with the pattern string.
- `ends_with: "pattern"`: Value must end with the pattern string.
- `regex: "pattern"`: Value must match the regular expression in pattern.

## Macros

A macro consists of one or more matching events, along with any preconditions that must be satisfied for the event to
match. It also includes one or more action to be executed if all filters and preconditions do match.

```yaml
matching_events:
  - ...
  
required_preconditions:
  - ...
  
actions:
  - ...
```

- `matching_events`: A list of events. If **at least one** of the events in this array matches an incoming event, it is
  satisfied. Further details about the contents of an event object later. This field is required.

- `required_preconditions`: A list of conditions that must **all** be satisfied for the macro to be executed. This field
  is optional.
  
- `actions`: A list of actions to be run, in sequence. This field is required.


### Events

An event fundamentally consists of a type of event, a data object with fields relevant to that type of event, and
optionally a list of required preconditions that only need to match for this event.

```yaml
type: midi
data:
  ...
required_preconditions:
  - ...
```

- `type`: This field instructs the program how to process the data contained in the `data` field. Since the software is
  mainly aimed at turning a MIDI device into a macro pad, this initially will only be "midi", but there is room for
  expansion to other types later.
- `data`: an object with fields relevant to this event type. The relevant ones for midi are specified later. This field
  is required.
- `required_preconditions`: A list of conditions that must **all** be satisfied for the macro to be executed. This field
  is optional. See Preconditions for detail.
  
#### MIDI events

The data object for matching a MIDI event looks as follows:

```yaml
message_type: note_on,
channel: 
  - 3
  - min: 5,
    max: 8
key: 32,
```

- `message_type`: Required, which MIDI message type to respond to. One event can only cover one type of MIDI message.
  Supported values are: 
  - `note_on`
  - `note_off`
  - `poly_aftertouch`
  - `control_change` 
  - `program_change`
  - `channel_aftertouch`
  - `pitch_bend_change`
  These must be in lowercase, exactly as written.
    
- `channel`: Optional. Which MIDI channel the event happens on. This is 0-based, so available channels are 0-15.
  See **value ranging** for how to specify matching values.
- `key`: Optional. Which key number is relevant to the event. See **value ranging**.
- `velocity`: Optional. How fast a key was pressed down or released. See **value ranging**.

The available properties depend on the value of `message_type`. Here is a comprehensive list:

- `note_on`
  - `channel` 0-15 inclusive
  - `key` 0-127 inclusive
  - `velocity` 0-127 inclusive
- `note_off`
  - `channel` 0-15 inclusive
  - `key` 0-127 inclusive
  - `velocity` 0-127 inclusive
- `poly_aftertouch`
  - `channel` 0-15 inclusive
  - `key` 0-127 inclusive
  - `value` 0-127 inclusive
- `control_change`
  - `channel` 0-15 inclusive
  - `control` 0-127 inclusive
  - `value` 0-127 inclusive
- `program_change`
  - `channel` 0-15 inclusive
  - `program` 0-127 inclusive
- `channel_aftertouch`
  - `channel` 0-15 inclusive
  - `value` 0-127 inclusive
- `pitch_bend_change`
  - `channel` 0-15 inclusive
  - `value` 0-16383 inclusive

#### Value ranging

The implementation for dealing with a specific event type determines the format of values that may be specified to match
values for that parameter.

##### MIDI

For MIDI value ranging, there are two distinct types of values.

- `message_type`: The value specified for this field must always be a single, quoted string, matching one supported
  values exactly. Any other value that cannot be interpreted is an error.
- All other fields: Everything else is numeric (positive integers). They may be omitted or set to `null` (without any
  quotes) to match any value.
  
Numeric fields can otherwise be set to a specific value directly to match only that one. For example: `key: 32` to
match one specific key. You may instead specify an array of values: `key: [32, 33, 34]` will match those 3 keys.

You may also specify a range: 

```yaml
velocity:
  min: 0
  max: 63
```
will match any velocity value from 0-63 inclusive.

Ranges can be open-ended: if you specify only `min`, then any value that is greater than or equal than the specified one
will match. If you specify only `max`, then any value smaller than or equal to the specified one will match.

Finally, you can mix the two:
```yaml
key: 
  - 12
  - 14
  - min: 32
    max: 44
```
will match keys 12, 14, and all the keys from 32 to 44 inclusive.

### Preconditions

A precondition is something that must be satisfied before a macro is allowed to run. These are based on state data
the program keeps track of.

A precondition is structured as follows:

```yaml
type: midi
invert: false
data:
  ...
```

- `type`: This field instructs the program how to process the data contained in the `data` field. Since the software is
mainly aimed at turning a MIDI device into a macro pad, this initially will only be "midi", but there is room for
expansion to other types later.
- `invert`: This inverts the matching of the conditions; if it normally matches but `invert` is set to `true`, it will
  be considered not a match, and vice versa. Optional field, defaulting to `false`.
- `data`: An object with fields relevant to the precondition type. These specify the condition that must be met.

#### MIDI Preconditions 

The program keeps track of notes that are currently on, as well as any control change and program change values.

This means that as soon as a note_on MIDI message is received, its data, if relevant, is available to match
preconditions.

The program remembers:

- "note_on" MIDI messages, keeping track per MIDI channel which notes are currently held down. Ass soon as a "note_off"
  message for the relevant key/channel is received, this state is removed.
- "control_change" MIDI messages, storing the last received value for any incoming control change on each channel
  indefinitely (well, until the program exits.)
- "program_change" MIDI messages, functioning the same way as "control_change"
- "pitch_bend_change" MIDI messages, also working the same way.

For example, a precondition that requires note 24 to be on on channel 1 looks as follows:

```yaml
type: midi
data:
  condition_type: note_on,
  channel: 1,
  key: 24
```

A precondition that requires control 42's value to be 64 or greater on channel 2 looks as follows:

```yaml
type: midi
data:
  condition_type: control,
  channel: 2,
  control: 42,
  value: 
    min: 64
```

Much like with MIDI events, the available parameters depend on the `condition_type` selected. Here is a comprehensive
list:

- `note_on`
  - `channel` 0-15 inclusive
  - `key` 0-127 inclusive
- `control`
  - `channel` 0-15 inclusive
  - `control` 0-127 inclusive
  - `value` 0-127 inclusive
- `program`
  - `channel` 0-15 inclusive
  - `program` 0-127 inclusive
- `pitch_bend`
  - `channel` 0-15 inclusive
  - `value` 0-16383 inclusive
  
Value ranging works the same way as it does for MIDI events, see **Value ranging** above.

### Actions

Actions describe what to do when an event and preconditions match one of the configured macros. 
Below all available actions and their arguments are described, but here's a list of them:

- key_sequence
- enter_text
- shell
- wait
- control

An action looks as follows:

```yaml
type: key_sequence,
data:
  ...
```

- `type`: specifies which kind of action. Its value determines what data fields are required and how it is executed.
  Must be one of `key_sequence`, `enter_text`, `shell`, `wait`, or `control` exactly.
- `data`: Object containing fields that differ based on `type`.

#### key_sequence

Key sequence actions allow you to enter a keyboard shortcut once or more. A full key sequence action looks as follows:

```yaml
type: key_sequence,
data: 
  sequence: "ctrl+shift+t",
  count: 2
  delay: 1500
```

- `sequence`: Required. A string representing the key combination. Key symbols are those from X Keysyms. A list may be
  found in the X11 source code file for [keysymdef.h](https://code.woboq.org/kde/include/X11/keysymdef.h.html).
  The symbols to use are the `XK_`-prefixed ones, without that prefix. To use multiple key sequences in a row, you can
  space-separate them like `"ctrl+shift+t Tab Tab Return"`
- `count`: Optional, defaults to 1. How many times to repeat entering this sequence.
- `delay`: Optional, defaults to 100. How many microseconds to wait between key presses. 
- `delay_ms`: Optional, shorthand for `delay` for larger values. How many milliseconds to wait between key presses.
  If both `delay` and `delay_ms` have valid values, the value for `delay` is used.

#### enter_text

Enter text actions allow you to type text as-written in response to an event, once or more. A full enter text action
follows:

```yaml
type: enter_text,
data: 
  text: "Hello world!",
  count: 1
  delay: 1500
```

- `text`: Required. String containing text exactly as you'd like it "typed" into the focused application.
- `count`: Optional, defaults to 1. How many times to repeat entering this sequence.
- `delay`: Optional, defaults to 100. How many microseconds to wait between key presses.
- `delay_ms`: Optional, shorthand for `delay` for larger values. How many milliseconds to wait between key presses.
  If both `delay` and `delay_ms` have valid values, the value for `delay` is used.

##### Shortened version

For both `key_sequence` and `enter_text`, you can specify the value directly for data to default to a count of 1.

Concretely, the following two are equivalent:

```yaml
type: enter_text,
data:
  text: "Hello world!"
  count: 1
  delay: 100
```

```yaml
type: enter_text
data: "Hello world!"
```

#### shell

Shell actions allow you to run arbitrary programs, with arbitrary arguments and environment variables. An example
follows:

```yaml
type: shell
data:
  command: "/usr/bin/echo"
  args:
    - "Hello"
    - "world"
  env_vars:
    key1: "val1"
    key2: "val2"
```

This action is equivalent to running this following in a bash prompt:

```bash
key1=val1 key2=val2 /bin/./echo "Hello" "world"
```

The fields are the following:

- `command`: Required. An absolute path to an executable file to run
- `args`: Optional. An array of arguments to pass to the command
- `env_vars`: Optional. An object with keys and values to set as environment variables

#### wait 

Wait actions insert a delay before continuing. They are helpful to allow some time between key sequences, to allow a
program processing them time to catch up.
An example follows:

```yaml
type: wait
data:
  duration: 2000 # 2 milliseconds
```

The following fields are available:

- `duration`: Duration of time to wait, expressed in microseconds
- `duration_ms`: Duration of time to wait, expressed in milliseconds

Either `duration` or `duration_ms` must be set. The value must be 0 or greater.
If both fields are present, `duration` is used, unless it contains a negative value.

##### Shortened version

You can specify duration in microseconds directly for the `data` field. The following two examples are equivalent.

```yaml
type: wait
data: 2000
```

```yaml
type: wait
data:
  duration: 2000
```

#### control

Control actions control the execution of mmpd itself. They must contain a control action in the data field. An example follows:

```yaml
type: control
data:
  action: exit
```

There is one field available in the `data` field:

- `action`:  What action to take, described in Control Actions below.

There is also a shorter form, here is an equivalent example:

```yaml
type: control
data: exit
```

##### Control Actions

The following control actions are available:

- `reload_macros`: Reloads the config file from disk and uses the updated macros found in it from there on.
  Does not change which MIDI device is listened to, and keeps any known state (such as keys held, control values)
  intact. If the config file couldn't be successfully read or parsed, mmpd will mention the error, but will keep running
  with data from the previously loaded valid configuration. If reloading is successful, but there are now no macros, it
  will exit.
- `restart`: Restarts mmpd with the same arguments that it was initially run with. It doesn't actually end the process,
  but it does re-initialize everything, including the MIDI device, blank state, etc. If anything about this is
  unsuccessful, mmpd will exit, just like it would if there are errors on a normal startup.
- `exit`: Immediately stops mmpd altogether.

---

#### Variables (NOT IMPLEMENTED YET)

Within actions, any data parameter that contains a string can use variables to insert some data from the event that
triggered the macro, or any state data kept.

For example, using the `enter_text` action, you can type the note number that was just pressed on the keyboard:

```yaml
# This example will not work, not yet implemented.
type: enter_text,
data:
  text: "%event.key%"
```

If the value specified is not available, this will output "none".

The syntax for accessing data this way is: `%variablename%`. A variable name uses dot notation for scoping.

For inserting info about a precondition in a string, access the `conditions` namespace. For example, accessing the last
known value of control 32 on midi channel 2: `%conditions.midi.channels[2].controls[32].value%`.

If you wish to use a literal `%` in a string, double it: `%%`.

##### Available data

- `event`: Any fields available from the event's `data` object. If it's a midi event, then they are all documented
  higher up.
  Examples:
  - `%event.key%`
  - `%event.channel`
  - `%event.velocity%`
  - `%event.control%`
  - `%event.value%`
- `conditions`: Any fields available in the state kept in memory, scoped per condition type:
  - `midi`: scope for midi preconditions
    - `channels`: List of MIDI channels, so a list 0-indexed list of 16 items, accessed with square brackets
      - `notes_on`: List of notes, so a list 0-indexed for all keys (128), accessed with square brackets. Each item
         returns a `"1"` or `"0"`
      - `controls`: List of controls, so a 0-index list of 128 items, accessed with square brackets. Each item is an
        integer ranged 0-127 inclusive or `"none"` if unknown.
      - `program`: 0-127 value, or "none" if unknown
      - `pitch_bend`: 0-16383 value or `"none"` if unknown.
  
Other top level namespaces may be added to expose more available data, or provide access to retrieving other data later.

---

## Full example of a config file

```yaml
version: 1

scopes:
  - window_class:
      contains: "gedit"
    macros: 
      - matching_events: 
          - type: midi
            data:
              message_type: note_on
              key: 33
        actions:
          - type: key_sequence,
            data:
              sequence: "ctrl+t"

global_macros:
  - matching_events:
    - type: midi
      data:
        message_type: note_on
        channel: 1
        key: 32
        velocity:
          min: 64
          
    required_preconditions: 
      - type: midi
        data:
          condition_type: control
          channel: 1
          control: 42
          value:
            max: 32
        
    actions: 
      - type: enter_text
        data:
          text: "Hello world!"
          count: 2
```

This example specifies a scope for gedit, a text editor, in which it will enter the sequence "ctrl+t" when key 33 is
pressed, regardless of channel or velocity.

Further, it specified a global macro, responding to a note_on event on MIDI channel 1, for specifically key 32, if
the velocity is 64 or higher. In order to run, control 42 on MIDI channel 1 must be known and set to no higher than 32.

It types the text "Hello world!" twice in the focused program.
