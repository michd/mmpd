use crate::keyboard_control::{self, KeyboardControlAdapter};
use crate::shell::{Shell, ShellImpl};
use std::{thread, time};
use regex::Regex;

/// Action run in response to a MIDI event
/// Any Action value can be run through ActionRunner::run.
#[derive(PartialEq, Debug)]
pub enum Action {
    /// Sends a key sequence 0 or more times
    /// Use this one for key combinations.
    /// The str argument specifies the key sequence, according to X Keysym notation.
    /// Per example "ctrl+shift+t": emulates pressing the "Ctrl", "Shift" and "t" keys at
    /// the same time.
    /// The number is how many times this key sequence should be entered.
    KeySequence {
        sequence: String,
        count: usize,
        delay: Option<u32>
    },

    /// Enters text as if you typed it on a keyboard
    /// Use this one for text exactly as in the string provided.
    /// The number is how many times this same string should be entered.
    EnterText {
        text: String,
        count: usize,
        delay: Option<u32>
    },

    /// Runs a program using the shell, allows running arbitrary programs.
    Shell {
        /// Absolute path to the program to run, without any arguments or options
        command: String,

        /// A list of arguments provided to the command. These end up space-separated.
        /// If one item includes spaces, that item will be surrounded by quotes so it's treated as
        /// one argument.
        args: Option<Vec<String>>,

        /// A list of key/value pairs with environment variables to be provided to the program
        env_vars: Option<Vec<(String, String)>>
    },

    /// Blocks the thread for a given amount of microseconds, to allow some previous action to be
    /// processed by the application that received input (if applicable)
    Wait {
        /// Duration to block the thread for, expressed in microseconds
        duration: u64
    },

    /// Controls the application itself via a ControlAction sub-action.
    Control(ControlAction)

    // This can be expanded upon
}

/// Action that controls application execution, allowing application control through macro
/// actions.
#[derive(PartialEq, Debug, Clone)]
pub enum ControlAction {
    /// Reread the configuration file from disk, and uses reloaded macros.
    /// Existing state in memory (keys held etc) is left as-is, and this won't change which
    /// MIDI device is being listened to, even if that changed in the config file.
    /// If you need to change MIDI devices, use `Restart` instead (which will also lose state).
    ReloadMacros,

    /// Restarts the program, mimicking killing the process and running it again with the same
    /// arguments.
    Restart,

    /// Exits the program entirely
    Exit
}

impl Action {
    /// Shorthand for creating the common simple form of a
    pub fn key_sequence(sequence: &str) -> Action {
        Action::KeySequence {
            sequence: sequence.to_string(),
            count: 1,
            delay: None
        }
    }

    /// Shorthand for creating the common simple form of Action::EnterText
    pub fn enter_text(text: &str) -> Action {
        Action::EnterText {
            text: text.to_string(),
            count: 1,
            delay: None
        }
    }
}

const DELAY_BETWEEN_KEYS_US: u32 = 100;

/// Struct to give access to running Actions
pub struct ActionRunner {
    kb_adapter: Box<dyn KeyboardControlAdapter>,
    shell_adapter: Box<dyn Shell>,
}

impl ActionRunner {
    /// Set up a new ActionRunner, relying on getting an adapter from keyboard_control.
    /// If no keyboard_control adapter can be obtained, returns None.
    pub fn new() -> Option<ActionRunner> {
        Some(ActionRunner {
            kb_adapter: keyboard_control::get_adapter()?,
            shell_adapter: Box::new(ShellImpl::new())
        })
    }

    /// Executes a given action based on action type
    /// If the action is an `Action:Control`, returns `Some(&control_action)`.
    /// In all other cases, returns `None`.
    pub fn run(&self, action: &Action) -> Option<ControlAction> {
        match action {
            Action::KeySequence { sequence, count, delay} => {
                self.run_key_sequence(sequence, *count, *delay);
            }

            Action::EnterText { text, count, delay } => {
                self.run_enter_text(text, *count, *delay)
            }

            Action::Shell { command, args, env_vars } => {
                self.run_shell(command, args.clone(), env_vars.clone());
            }

            Action::Wait { duration } => {
                self.run_wait(*duration);
            }

            Action::Control(control_action) => {
                return Some(control_action.clone());
            }
        }

        return None;
    }

    fn run_key_sequence(&self, sequence: &str, count: usize, delay: Option<u32>) {
        let separator = Regex::new(r"\s+").expect("Invalid space regex");
        let sequences: Vec<&str> = separator.split(sequence).into_iter().collect();

        for _ in 0..count {
            for seq in &sequences {
                // Note: swallowing potential error
                // TODO: expose these errors all the way up
                // TODO: stop running sequence as soon as an error is encountered
                let _ = self.kb_adapter.send_keysequence(
                    seq,
                    delay.unwrap_or(DELAY_BETWEEN_KEYS_US)
                );
            }
        }
    }

    fn run_enter_text(&self, text: &str, count: usize, delay: Option<u32>) {
        for _ in 0..count {
            // Note: swallowing potential error
            // TODO: expose errors all the way up
            // TODO: stop entering text as soon as an error is encountered
            let _ = self.kb_adapter.send_text(
                text,
                delay.unwrap_or(DELAY_BETWEEN_KEYS_US)
            );
        }
    }

    fn run_shell(
        &self,
        command: &str,
        args: Option<Vec<String>>,
        env_vars: Option<Vec<(String, String)>>
    ) {
        // TODO: it would be good to be able to substitute certain patterns in any of the strings
        // used in these commands. Substitutable values would essentially include any parameter that
        // was involved in leading to this action being run. That is, any parameters of the
        // MidiMessage, and perhaps access to the whole of the Midi state being stored in memory.
        // This needs further working out to get sensible var names.

        self.shell_adapter.execute(command, args, env_vars);
    }

    fn run_wait(&self, duration: u64) {
        thread::sleep(time::Duration::from_micros(duration));
    }
}

#[cfg(test)]
mod tests {
    use crate::macros::actions::{ActionRunner, Action, DELAY_BETWEEN_KEYS_US, ControlAction};
    use crate::keyboard_control::adapters::MockKeyboardControlAdapter;
    use crate::shell::{Shell, MockShell};
    use mockall::predicate::eq;
    use crate::keyboard_control::KeyboardControlAdapter;

    /// Helper struct to make setting up an ActionRunner for tests slightly
    /// less of a hassle, having to provide only te dependencies that we want to
    /// look into.
    struct ActionRunnerBuilder {
        kb_adapter: Option<Box<dyn KeyboardControlAdapter>>,
        shell_adapter: Option<Box<dyn Shell>>
    }

    impl ActionRunnerBuilder {
        fn new() -> ActionRunnerBuilder {
            ActionRunnerBuilder {
                kb_adapter: None,
                shell_adapter: None
            }
        }

        fn set_keyboard_adapter(mut self, kb_adapter: Box<dyn KeyboardControlAdapter>) -> Self {
            self.kb_adapter = Some(kb_adapter);
            self
        }

        fn set_shell_adapter(mut self, shell_adapter: Box<dyn Shell>) -> Self {
            self.shell_adapter = Some(shell_adapter);
            self
        }

        fn into_runner(self) -> ActionRunner {
            ActionRunner {
                kb_adapter: self.kb_adapter.unwrap_or(Box::new(MockKeyboardControlAdapter::new())),
                shell_adapter: self.shell_adapter.unwrap_or(Box::new(MockShell::new()))
            }
        }
    }

    #[test]
    fn runs_single_key_sequence() {
        let mut mock_keyb_adapter = MockKeyboardControlAdapter::new();

        mock_keyb_adapter.expect_send_keysequence()
            .with(eq("ctrl+alt+delete"), eq(DELAY_BETWEEN_KEYS_US))
            .times(1)
            .return_const(());

        let runner = ActionRunnerBuilder::new()
            .set_keyboard_adapter(Box::new(mock_keyb_adapter))
            .into_runner();

        let result = runner.run(&Action::key_sequence("ctrl+alt+delete"));

        assert!(result.is_none());
    }

    #[test]
    fn runs_repeated_key_sequences() {
        let mut mock_keyb_adapter = MockKeyboardControlAdapter::new();

        mock_keyb_adapter.expect_send_keysequence()
            .with(eq("Tab"), eq(DELAY_BETWEEN_KEYS_US))
            .times(3)
            .return_const(());

        let runner = ActionRunnerBuilder::new()
            .set_keyboard_adapter(Box::new(mock_keyb_adapter))
            .into_runner();

        let result = runner.run(&Action::KeySequence {
            sequence: "Tab".to_string(),
            count: 3,
            delay: None
        });

        assert!(result.is_none());
    }

    #[test]
    fn runs_space_separated_key_sequences() {
        let mut mock_keyb_adapter = MockKeyboardControlAdapter::new();

        mock_keyb_adapter.expect_send_keysequence()
            .with(eq("ctrl+t"), eq(DELAY_BETWEEN_KEYS_US))
            .times(1)
            .return_const(());

        mock_keyb_adapter.expect_send_keysequence()
            .with(eq("Tab"), eq(DELAY_BETWEEN_KEYS_US))
            .times(2)
            .return_const(());

        mock_keyb_adapter.expect_send_keysequence()
            .with(eq("Return"), eq(DELAY_BETWEEN_KEYS_US))
            .times(1)
            .return_const(());

        let runner = ActionRunnerBuilder::new()
            .set_keyboard_adapter(Box::new(mock_keyb_adapter))
            .into_runner();

        let result = runner.run(&Action::KeySequence {
            // Should deal with arbitrary amounts of space characters in between sequences
            sequence: "ctrl+t Tab   Tab  Return".to_string(),
            count: 1,
            delay: None
        });

        assert!(result.is_none());
    }

    #[test]
    fn runs_repeated_space_separated_key_sequences() {
        let mut mock_keyb_adapter = MockKeyboardControlAdapter::new();

        mock_keyb_adapter.expect_send_keysequence()
            .with(eq("ctrl+t"), eq(DELAY_BETWEEN_KEYS_US))
            .times(3)
            .return_const(());

        mock_keyb_adapter.expect_send_keysequence()
            .with(eq("Tab"), eq(DELAY_BETWEEN_KEYS_US))
            .times(6)
            .return_const(());

        mock_keyb_adapter.expect_send_keysequence()
            .with(eq("Return"), eq(DELAY_BETWEEN_KEYS_US))
            .times(3)
            .return_const(());

        let runner = ActionRunnerBuilder::new()
            .set_keyboard_adapter(Box::new(mock_keyb_adapter))
            .into_runner();

        let result = runner.run(&Action::KeySequence {
            // Should deal with arbitrary amounts of space characters in between sequences
            sequence: "ctrl+t Tab   Tab  Return".to_string(),
            count: 3,
            delay: None
        });

        assert!(result.is_none());
    }

    #[test]
    fn runs_single_send_text() {
        let mut mock_keyb_adapter = MockKeyboardControlAdapter::new();

        mock_keyb_adapter.expect_send_text()
            .with(eq("hello"), eq(DELAY_BETWEEN_KEYS_US))
            .times(1)
            .return_const(());

        let runner = ActionRunnerBuilder::new()
            .set_keyboard_adapter(Box::new(mock_keyb_adapter))
            .into_runner();

        let result = runner.run(&Action::enter_text("hello"));

        assert!(result.is_none());
    }

    #[test]
    fn runs_repeated_send_text() {
        let mut mock_keyb_adapter = MockKeyboardControlAdapter::new();

        mock_keyb_adapter.expect_send_text()
            .with(eq("hello"), eq(DELAY_BETWEEN_KEYS_US))
            .times(3)
            .return_const(());

        let runner = ActionRunnerBuilder::new()
            .set_keyboard_adapter(Box::new(mock_keyb_adapter))
            .into_runner();

        let result = runner.run(&Action::EnterText {
            text: "hello".to_string(),
            count: 3,
            delay: None
        });

        assert!(result.is_none());
    }

    #[test]
    fn runs_shell_actions() {
        let mut mock_shell = MockShell::new();

        // TODO: Currently this checks only if parameters are passed through as they came.
        // Later we will want to process some input event-related variables by doing string
        // substitution in arguments / env vars. At that point a unit tests for this
        // functionality becomes actually useful.

        // TODO: this format of test with Mockall does not show very useful
        // output when it fails; room for improvement.
        mock_shell.expect_execute()
            .withf(|cmd, args, env_vars| {
                let expected_cmd = "test_cmd";
                let expected_args = Some(vec!["arg1".to_string(), "arg2".to_string()]);
                let expected_env_vars = Some(vec![
                    ("key1".to_string(), "val1".to_string()),
                    ("key2".to_string(), "val2".to_string())
                ]);

                cmd == expected_cmd
                    && do_opt_vecs_match(args, &expected_args)
                    && do_opt_vecs_match(env_vars, &expected_env_vars)
            })
            .times(1)
            .return_const(());

        let runner = ActionRunnerBuilder::new()
            .set_shell_adapter(Box::new(mock_shell))
            .into_runner();

        let result = runner.run(&Action::Shell {
            command: "test_cmd".to_string(),
            args: Some(vec!["arg1".to_string(), "arg2".to_string()]),
            env_vars: Some(vec![
                ("key1".to_string(), "val1".to_string()),
                ("key2".to_string(), "val2".to_string())
            ])
        });

        assert!(result.is_none());
    }

    // TODO: way to test `Action::Wait`. It's a very straightforward one, but testing is good.
    // I don't know if there's a way to mock thread::sleep somehow without doing a whole adapter
    // thing for it again like Action::Shell.

    #[test]
    fn passes_through_control_actions() {
        let action = Action::Control(ControlAction::Exit);

        let runner = ActionRunnerBuilder::new().into_runner();

        let result = runner.run(&action);

        assert_eq!(result, Some(ControlAction::Exit));
    }

    // Helper function to see if two vectors are identical
    // TODO: perhaps move to some test util module.
    fn do_vecs_match<T: PartialEq>(a: &Vec<T>, b: &Vec<T>) -> bool {
        if a.len() != b.len() {
            return false;
        }

        for (a, b) in a.iter().zip(b.iter()) {
            if a != b {
                return false
            }
        }

        true
    }

    fn do_opt_vecs_match<T: PartialEq>(a: &Option<Vec<T>>, b: &Option<Vec<T>>) -> bool {
        if let None = a {
            return if let None = b {
                true
            } else {
                false
            }
        }

        if let None = b {
            return false
        }

        do_vecs_match(&a.as_ref().unwrap(), &b.as_ref().unwrap())
    }
}