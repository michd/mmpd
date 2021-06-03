use std::process::Command;

#[cfg(test)]
use mockall::automock;

/// Wrapper for executing shell commands, really a single-method
/// interface for std::process::Command. Exists mainly to facilitate
/// mocking in unit tests.
#[cfg_attr(test, automock)]
pub(crate) trait Shell {
    fn execute(
        &self,
        command: &str,
        args: Option<Vec<String>>,
        env_vars: Option<Vec<(String, String)>>
    );
}

pub(crate) struct ShellImpl {}

impl ShellImpl {
    pub fn new() -> impl Shell {
        ShellImpl {}
    }
}

impl Shell for ShellImpl {
    fn execute(
        &self,
        command: &str,
        args: Option<Vec<String>>,
        env_vars: Option<Vec<(String, String)>>
    ) {
        let mut cmd = Command::new(command);

        // Attach any arguments
        if let Some(args) = args {
            for arg in args.iter() {
                cmd.arg(arg);
            }
        }

        // Attach any environment expressions
        if let Some(env_vars) = env_vars {
            for (env_key, env_val) in env_vars {
                cmd.env(env_key, env_val);
            }
        }

        // Run
        let _ = cmd.status();
    }
}