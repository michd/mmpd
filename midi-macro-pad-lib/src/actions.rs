use crate::keyboard_control::{self, KeyboardControlAdapter};
use crate::shell::{Shell, ShellImpl};
use mockall::predicate::*;

/// Action run in response to a MIDI event
/// Any Action value can be run through ActionRunner::run.
pub enum Action<'a> {
    /// Sends a key sequence 0 or more times
    /// Use this one for key combinations.
    /// The str argument specifies the key sequence, according to X Keysym notation.
    /// Per example "ctrl+shift+t": emulates pressing the "Ctrl", "Shift" and "t" keys at
    /// the same time.
    /// The number is how many times this key sequence should be entered.
    KeySequence(&'a str, usize),

    /// Enters text as if you typed it on a keyboard
    /// Use this one for text exactly as in the string provided.
    /// The number is how many times this same string should be entered.
    EnterText(&'a str, usize),

    /// Runs a program using the shell, allows running arbitrary programs.
    Shell {
        /// Absolute path to the program to run, without any arguments or options
        command: &'a str,

        /// A list of arguments provided to the command. These end up space-separated.
        /// If one item includes spaces, that item will be surrounded by quotes so it's treated as
        /// one argument.
        args: Option<Vec<&'a str>>,

        /// A list of key/value pairs with environment variables to be provided to the program
        env_vars: Option<Vec<(&'a str, &'a str)>>
    },

    /// A list of actions to be run in the order specified, to allow executing several different
    /// ones for more complex things.
    Combination(Vec<Action<'a>>),

    // This can be expanded upon
}

const DELAY_BETWEEN_KEYS_US: u32 = 100;

/// Struct to give access to running Actions
pub struct ActionRunner {
    kb_adapter: Box<dyn KeyboardControlAdapter>,
    shell_adapter: Box<dyn Shell>
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

    fn test_new(
        kb_adapter: Box<dyn KeyboardControlAdapter>,
        shell_adapter: Box<dyn Shell>
    ) -> Option<ActionRunner> {
        Some(ActionRunner {
            kb_adapter,
            shell_adapter
        })
    }

    /// Executes a given action based on action type
    pub fn run(&self, action: &Action) {
        match action {
            Action::KeySequence(sequence, count) => {
                self.run_key_sequence(sequence, *count);
            },

            Action::EnterText(text, count) => {
                self.run_enter_text(text, *count);
            },

            Action::Shell { command, args, env_vars } => {
                self.run_shell(command, args.clone(), env_vars.clone());
            },

            Action::Combination(actions) => {
                for action in actions {
                    self.run(action);
                }
            },
        }
    }

    fn run_key_sequence(&self, sequence: &str, count: usize) {
        for _ in 0..count {
            self.kb_adapter.send_keysequence(sequence, DELAY_BETWEEN_KEYS_US);
        }
    }

    fn run_enter_text(&self, text: &str, count: usize) {
        for _ in 0..count {
            self.kb_adapter.send_text(text, DELAY_BETWEEN_KEYS_US);
        }
    }

    fn run_shell(
        &self,
        command: &str,
        args: Option<Vec<&str>>,
        env_vars: Option<Vec<(&str, &str)>>
    ) {
        // TODO: it would be good to be able to substitute certain patterns in any of the strings
        // used in these commands. Substitutable values would essentially include any parameter that
        // was involved in leading to this action being run. That is, any parameters of the
        // MidiMessage, and perhaps access to the whole of the Midi state being stored in memory.
        // This needs further working out to get sensible var names.

        self.shell_adapter.execute(command, args, env_vars);
    }
}

#[cfg(test)]
mod tests {
    use crate::actions::{ActionRunner, Action, DELAY_BETWEEN_KEYS_US};
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
            ActionRunner::test_new(
                self.kb_adapter.unwrap_or(Box::new(MockKeyboardControlAdapter::new())),
                self.shell_adapter.unwrap_or(Box::new(MockShell::new()))
            ).unwrap()
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

        runner.run(&Action::KeySequence("ctrl+alt+delete", 1));
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

        runner.run(&Action::KeySequence("Tab", 3));
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

        runner.run(&Action::EnterText("hello", 1));
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

        runner.run(&Action::EnterText("hello", 3));
    }

    #[test]
    fn runs_combination_action() {
        let mut mock_keyb_adapter = MockKeyboardControlAdapter::new();

        mock_keyb_adapter.expect_send_keysequence()
            .with(eq("ctrl+shift+Tab"), eq(DELAY_BETWEEN_KEYS_US))
            .times(2)
            .returning(|_, _| ());

        mock_keyb_adapter.expect_send_text()
            .with(eq("hello"), eq(DELAY_BETWEEN_KEYS_US))
            .times(3)
            .returning(|_, _| ());

        let runner = ActionRunnerBuilder::new()
            .set_keyboard_adapter(Box::new(mock_keyb_adapter))
            .into_runner();

        runner.run(&Action::Combination(vec![
            Action::KeySequence("ctrl+shift+Tab", 2),
            Action::EnterText("hello", 3)
        ]));
    }

    #[test]
    fn runs_shell_actions() {
        let mut mock_shell = MockShell::new();

        // TODO: Currently this checks only if paramaters are passed through as they came.
        // Later we will want to process some input event-related variables by doing string
        // substitution in arguments / env vars. At that point a unit tests for this
        // functionality becomes actually useful.

        // TODO: this format of test with Mockall does not show very useful
        // output when it fails; room for improvement.
        mock_shell.expect_execute()
            .withf(|cmd, args, env_vars| {
                let expected_cmd = "test_cmd";
                let expected_args = Some(vec!["arg1", "arg2"]);
                let expected_env_vars = Some(vec![("key1", "val1"), ("key2", "val2")]);

                cmd == expected_cmd
                    && do_opt_vecs_match(args, &expected_args)
                    && do_opt_vecs_match(env_vars, &expected_env_vars)
            })
            .times(1)
            .return_const(());

        let runner = ActionRunnerBuilder::new()
            .set_shell_adapter(Box::new(mock_shell))
            .into_runner();

        runner.run(&Action::Shell {
            command: "test_cmd",
            args: Some(vec!["arg1", "arg2"]),
            env_vars: Some(vec![("key1", "val1"), ("key2", "val2")])
        });
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