#[derive(PartialEq, Debug)]
struct Token {
    name: String,
    kind: TokenKind,
}

#[derive(PartialEq, Debug)]
enum TokenKind {
    Leaf,
    ArrayIndex(usize),
    Namespace,
    FunctionCall(Vec<String>)
}

#[derive(Debug)]
enum VariableError {
    Parser(String),
    Other(String),
}

enum ParserState {
    Name, // Expect valid characters to build a name, or any that start array of function call
    Array, // Expect consecutive decimal digits or ']'
    FunctionCall, // Expect basically anything; special meaning: `,` `)` unless in double quotes
                  // Double quotes can be escaped with a backslash, and a backslash can be escaped
                  // with another
    AfterToken,    // A token has ended (like after ] for array, or ) for function call. If
                  // Something further comes after it should be .-separated
    End, // End of string reached
}

struct Parser {
    var_str: String,
    index: usize,
    nodes: Vec<Token>,
    current_name: String,
    current_array_index_str: String,
    state: ParserState,
}

impl Parser {
    fn new(var_str: &str) -> Parser {
        Parser {
            var_str: var_str.trim().to_string(),
            index: 0,
            nodes: vec![],
            current_name: "".to_string(),
            current_array_index_str: "".to_string(),
            state: ParserState::Name
        }
    }

    fn parse(mut self) -> Result<Vec<Token>, VariableError> {
        if self.var_str.is_empty() {
            return Err(VariableError::Parser(
                "No valid characters found in variable name".to_string()
            ));
        }

        loop {
            let chars_read = match self.state {
                ParserState::Name => self.read_name_chars()?,
                ParserState::Array => self.read_array_chars()?,
                ParserState::FunctionCall => self.read_function_call_chars()?,
                ParserState::AfterToken => self.read_after_token_chars()?,
                ParserState::End => break
            };

            self.index += chars_read;
        }

        Ok(self.nodes)
    }

    fn read_name_chars(&mut self) -> Result<usize, VariableError> {
        for (i, c) in self.var_str[self.index..].chars().enumerate() {
            match c {
                // Valid token name characters
                'a'..='z' | 'A'..='Z' | '0'..='9' | '_' => {
                    self.current_name.push_str(c.to_string().as_str())
                }

                '.' => {
                    self.nodes.push(
                        Token { name: self.current_name.to_owned(), kind: TokenKind::Namespace }
                    );

                    self.current_name = "".to_string();
                    self.state = ParserState::Name;

                    self.state = if self.index >= self.var_str.len() {
                        ParserState::End
                    } else {
                        ParserState::Name
                    };

                    return Ok(i + 1);
                }

                '[' => {
                    self.state = ParserState::Array;
                    return Ok(i + 1);
                },

                '(' => {
                    self.state = ParserState::FunctionCall;
                    return Ok(i + 1);
                },

                _ => {
                    return Err(VariableError::Parser(format!(
                        "Invalid character '{}' at location {} in variable notation:\n{}",
                        c.to_string(),
                        self.index + i,
                        self.var_str
                    )));
                }
            }
        }

        self.state = ParserState::End;

        if self.current_name.is_empty() {
            Err(VariableError::Parser(format!(
                "Unexpected end of variable notation string; expecting a name. \
                    Variable notation:\n{}",
                self.var_str
            )))
        } else {
            self.nodes.push(
                Token { name: self.current_name.to_owned(), kind: TokenKind::Leaf }
            );

            Ok(self.var_str.len() - self.index)
        }
    }

    fn read_array_chars(&mut self) -> Result<usize, VariableError> {
        for (i, c) in self.var_str[self.index..].chars().enumerate() {
            match c {
                '0'..='9' => {
                    self.current_array_index_str.push_str(c.to_string().as_str())
                }

                ']' => {
                    if self.current_array_index_str.is_empty() {
                        return Err(VariableError::Parser(format!(
                            "Missing array index at location {} in variable notation:\n{}",
                            self.index + i,
                            self.var_str
                        )));
                    }

                    return match usize::from_str_radix(self.current_array_index_str.as_str(), 10) {
                        Ok(arr_index) => {
                            self.nodes.push(
                                Token {
                                    name: self.current_name.to_owned(),
                                    kind: TokenKind::ArrayIndex(arr_index)
                                }
                            );

                            self.current_name = "".to_string();
                            self.current_array_index_str = "".to_string();
                            self.state = ParserState::AfterToken;
                            Ok(i + 1)
                        }

                        Err(e) => {
                            Err(VariableError::Parser(format!(
                                "Failed to parse array index '{}': {}. Found at location {} in \
                                variable notation:\n{}",
                                self.current_array_index_str,
                                e.to_string(),
                                self.index + i - 1,
                                self.var_str
                            )))
                        }
                    }
                }

                _ => {
                    return Err(VariableError::Parser(format!(
                        "Invalid character '{}' at location {} in variable notation:\n{}",
                        c.to_string(),
                        self.index + i,
                        self.var_str
                    )));
                }
            }
        }

        Err(VariableError::Parser(format!(
            "Unexpected end of variable notation string; expecting decimal digits or ']'. \
                Variable notation:\n{}",
            self.var_str
        )))
    }

    // Todo split this up into its own parser further
    fn read_function_call_chars(&mut self) -> Result<usize, VariableError> {
        let mut is_escaping = false;
        let mut is_in_quotes = false;
        let mut current_arg = "".to_string();

        let mut expect_comma_or_paren = false;

        let mut args: Vec<String> = vec![];

        const WHITESPACE_CHARS: [char; 10] = [
            ' ', // Space
            '\t', // Tab
            '\n', // Newline
            '\r', // Carriage return,
            '\u{000B}', // Vertical tab
            '\u{0085}', // Next line
            '\u{200E}', // Left-to-right mark
            '\u{200F}', // Right-to-left mark
            '\u{2028}', // Line separator,
            '\u{2029}', // Paragraph separator
        ];

        for (i, c) in self.var_str[self.index..].chars().enumerate() {
            if expect_comma_or_paren {
                match c {
                    ',' => {
                        expect_comma_or_paren = false;
                        continue
                    }

                    ')' => {
                        self.nodes.push(
                            Token {
                                name: self.current_name.to_owned(),
                                kind: TokenKind::FunctionCall(args)
                            }
                        );

                        self.current_name = "".to_string();
                        self.state = ParserState::AfterToken;
                        return Ok(i + 1);
                    }

                    _ if WHITESPACE_CHARS.contains(&c) => {
                        // Ignore.
                        continue;
                    }

                    _ => {
                        // Invalid character at this point.
                        eprintln!(
                            "erring, is_in_quotes: {:?}, is_escaping: {:?}, current_arg: {:?}, args: {:?}",
                            is_in_quotes,
                            is_escaping,
                            current_arg,
                            args
                        );
                        return Err(VariableError::Parser(format!(
                            "Invalid character '{}' at location {} in function args in variable \
                            notation:\n{}\nExpected whitespace, ',', or ')' at this point.",
                            c.to_string(),
                            self.index + i,
                            self.var_str
                        )))
                    }
                }
            }

            match c {
                '"' => {
                    if is_in_quotes {
                        if is_escaping {
                            // If escaping, use literal '"' character as part of argument value
                            current_arg.push_str(c.to_string().as_str());
                            is_escaping = false
                        } else {
                            // Otherwise, close quotes, end of argument value
                            args.push(current_arg.to_owned());
                            current_arg = "".to_string();
                            is_in_quotes = false;

                            // After an argument ends, we only expect limited possible values,
                            // so this becomes a special case for the next char
                            expect_comma_or_paren = true;
                        }

                        continue;
                    }

                    if !current_arg.is_empty() {
                        // Error, a quote mid-argument
                        eprintln!(
                            "erring, is_in_quotes: {:?}, is_escaping: {:?}, current_arg: {:?}, args: {:?}",
                            is_in_quotes,
                            is_escaping,
                            current_arg,
                            args
                        );
                        return Err(VariableError::Parser(format!(
                            "Unexpected '\"' quote character mid-argument at location {} in \
                             function args in variable notation:\n{}",
                            self.index + i,
                            self.var_str
                        )));
                    }

                    // Nothing in current argument yet, start argument by being in quotes.
                    is_in_quotes = true;
                }

                '\\' => {
                    if is_in_quotes {
                        if is_escaping {
                            // If we're already escaping, this adds the literal backslash char
                            current_arg.push_str(c.to_string().as_str());
                            is_escaping = false;
                        } else {
                            // Start an escape if we're in quotes
                            is_escaping = true
                        }
                    } else {
                        // If not in quotes, consider it a normal character
                        current_arg.push_str(c.to_string().as_str());
                    }
                }

                ',' => {
                    if current_arg.is_empty() && !is_in_quotes {
                        // Don't expect a comma when we don't have an ongoing current argument
                        eprintln!(
                            "erring, is_in_quotes: {:?}, is_escaping: {:?}, current_arg: {:?}, args: {:?}",
                            is_in_quotes,
                            is_escaping,
                            current_arg,
                            args
                        );
                        return Err(VariableError::Parser(format!(
                            "Unexpected ',' at location {} in function function args in \
                            variable notation:\n{}",
                            self.index + i,
                            self.var_str
                        )));
                    }

                    if is_in_quotes {
                        current_arg.push_str(c.to_string().as_str());
                    } else if !current_arg.is_empty() {
                        args.push(current_arg.to_owned());
                        current_arg = "".to_string();
                    }
                }

                ')' => {
                    if !is_in_quotes {
                        // If not in quotes, then this ends the argument list and the function
                        // call syntax
                        if !current_arg.is_empty() {
                            // If we had an ongoing argument, add it to the list first
                            args.push(current_arg.to_owned());
                        }

                        // Finish up creating the token and push it
                        self.nodes.push(Token {
                            name: self.current_name.to_owned(),
                            kind: TokenKind::FunctionCall(args)
                        });

                        self.current_name = "".to_string();
                        self.state = ParserState::AfterToken;
                        return Ok(i + 1);
                    }

                    // Otherwise, add to current arg
                    current_arg.push_str(c.to_string().as_str());
                }

                _ if WHITESPACE_CHARS.contains(&c) => {
                   if is_in_quotes {
                       current_arg.push_str(c.to_string().as_str());
                   }
                }

                _ => {
                    // Any other characters, add to current_arg
                    current_arg.push_str(c.to_string().as_str());
                }
            }

            // If the is_escaping flag was still set when the character just seen was _not_ a
            // backslash, this flag should be turned off again.
            if is_escaping && c != '\\' {
                is_escaping = false;
            }
        }

        // Reached end of var_str
        // Shouldn't get here at all, expecting a ) to finish things, which is above.
        eprintln!(
            "erring, is_in_quotes: {:?}, is_escaping: {:?}, current_arg: {:?}, args: {:?}",
            is_in_quotes,
            is_escaping,
            current_arg,
            args
        );

        Err(VariableError::Parser(format!(
            "Unexpected end of variable notation string during function call syntax. \
            Variable notation:\n{}",
            self.var_str
        )))
    }

    fn read_after_token_chars(&mut self) -> Result<usize, VariableError> {
        match self.var_str.chars().nth(self.index) {
            Some('.') => {
                self.state = ParserState::Name;
                Ok(1)
            }

            Some(c) => {
                Err(VariableError::Parser(format!(
                    "Invalid character '{}' (expected '.') at location {} in variable \
                        notation:\n{}",
                    c.to_string(),
                    self.index,
                    self.var_str
                )))
            }

            None => {
                self.state = ParserState::End;
                Ok(0)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::variables::{Parser, Token, TokenKind};

    #[test]
    fn parses_a_single_leaf_node() {
        assert_eq!(
            Parser::new("leaf_token").parse().unwrap(),
            vec![Token { name: "leaf_token".to_string(), kind: TokenKind::Leaf }]
        );

        assert!(
            Parser::new("!!invalid name!!").parse().is_err()
        )
    }

    #[test]
    fn parses_a_leaf_after_a_namespace() {
        assert_eq!(
            Parser::new("my_namespace.my_leaf").parse().unwrap(),
            vec![
                Token { name: "my_namespace".to_string(), kind: TokenKind::Namespace },
                Token { name: "my_leaf".to_string(), kind: TokenKind::Leaf }
            ]
        );

        // Wrong separator
        assert!(
            Parser::new("my_namespace|my_leaf").parse().is_err()
        );
    }

    #[test]
    fn parses_multiple_namespaces() {
        assert_eq!(
            Parser::new("my_namespace.my_sub_namespace.my_leaf").parse().unwrap(),
            vec![
                Token { name: "my_namespace".to_string(), kind: TokenKind::Namespace },
                Token { name: "my_sub_namespace".to_string(), kind: TokenKind::Namespace },
                Token { name: "my_leaf".to_string(), kind: TokenKind::Leaf }
            ]
        );
    }

    #[test]
    fn parses_an_array_index_token() {
        assert_eq!(
            Parser::new("my_namespace.arr[823]").parse().unwrap(),
            vec![
                Token { name: "my_namespace".to_string(), kind: TokenKind::Namespace },
                Token { name: "arr".to_string(), kind: TokenKind::ArrayIndex(823) }
            ]
        );

        // Unclosed array notation
        assert!(
            Parser::new("my_namespace.arr[823").parse().is_err()
        );

        // Non-digits in index
        assert!(
            Parser::new("my_namespace.arr[INVALID]").parse().is_err()
        );
    }

    #[test]
    fn test_helper_correctly_makes_function_call_token() {
        assert_eq!(
            Token {
                name: "fun".to_string(),
                kind: TokenKind::FunctionCall(vec![
                    "arg1".to_string(),
                    "arg2".to_string()
                ])
            },
            func_token("fun", vec!["arg1", "arg2"])
        )
    }

    #[test]
    fn parses_a_function_call() {
        // Oh aren't all these fun
        assert_eq!(
            Parser::new("my_namespace.func(abc, 123, sup)").parse().unwrap(),
            vec![
                Token { name: "my_namespace".to_string(), kind: TokenKind::Namespace },
                func_token("func", vec!["abc", "123", "sup"])
            ]
        );

        assert_eq!(
            Parser::new(r#"fun("ab")"#).parse().unwrap(),
            vec![func_token("fun", vec!["ab"])]
        );

        assert_eq!(
            Parser::new(r#"fun("abc",  foo   )"#).parse().unwrap(),
            vec![func_token("fun", vec!["abc", "foo"])]
        );

        assert_eq!(
            Parser::new(r#"fun(arg1,"arg2",arg3)"#).parse().unwrap(),
            vec![func_token("fun", vec!["arg1", "arg2", "arg3"])]
        );

        assert_eq!(
            Parser::new(r#"fun(arg1, "arg number two", "arg \"three\"")"#).parse().unwrap(),
            vec![func_token("fun", vec!["arg1", "arg number two", r#"arg "three""#])]
        );

        assert_eq!(
            Parser::new(r#"fun(ar\g1   ,"ar\g2",  arg3)"#).parse().unwrap(),
            vec![func_token("fun", vec![r#"ar\g1"#, r#"arg2"#, "arg3"])]
        );

        assert_eq!(
            Parser::new(r#"fun(ar\\g1, "arg\\2", arg3)"#).parse().unwrap(),
            vec![func_token("fun", vec![r#"ar\\g1"#, r#"arg\2"#, "arg3"])]
        );

        assert_eq!(
            Parser::new("fun_1_2()").parse().unwrap(),
            vec![func_token("fun_1_2", vec![])]
        );

        assert_eq!(
            Parser::new("fun(        )").parse().unwrap(),
            vec![func_token("fun", vec![])]
        );

        assert_eq!(
            Parser::new(r#"fun(*, C3)"#).parse().unwrap(),
            vec![func_token("fun", vec!["*", "C3"])]
        );

        assert_eq!(
            Parser::new(
                r#"namespace.fun1(f1_arg1, "f1 arg \\two\\").fun2("f2 \"arg\" one", f2_arg_two)"#
            ).parse().unwrap(),
            vec![
                Token { name: "namespace".to_string(), kind: TokenKind::Namespace },
                func_token("fun1", vec!["f1_arg1", r#"f1 arg \two\"#]),
                func_token("fun2", vec![r#"f2 "arg" one"#, "f2_arg_two"])
            ]
        );

        // Invalid characters
        assert!(
            Parser::new("my_namespace.func(abc), 123)").parse().is_err()
        );

        // Unclosed parentheses
        assert!(
            Parser::new("fun(foo").parse().is_err()
        );

        // Unclosed quotes, which means it's interpreted as missing quote /
        // missing closing parentheses
        assert!(
            Parser::new(r#"fun("foo)"#).parse().is_err()
        );

        // Uncloses parentheses, the opening paren at the end is irrelevant
        assert!(
            Parser::new(r#"fun("foo", bar("#).parse().is_err()
        );

        // Unclosed parenthesis due to still being in quoted string due to escaped quote
        assert!(
            Parser::new(r#"fun("arg\")"#).parse().is_err()
        );

        // Unexpected comma
        assert!(
            Parser::new("fun(,foo)").parse().is_err()
        );

        // Unexpected character after closing quotes for an arg; expects a comma
        assert!(
            Parser::new(r#"fun("abc" asdf"#).parse().is_err()
        );
    }

    #[test]
    fn parses_realistic_examples() {
        // Accessing a field from the event that triggered this action
        assert_eq!(
            Parser::new("event.key").parse().unwrap(),
            vec![
                Token { name: "event".to_string(), kind: TokenKind::Namespace },
                Token { name: "key".to_string(), kind: TokenKind::Leaf },
            ]
        );

        // Accessing a specific MIDI control value from state
        assert_eq!(
            Parser::new("state.midi.channels[3].controls[15].value").parse().unwrap(),
            vec![
                Token { name: "state".to_string(), kind: TokenKind::Namespace },
                Token { name: "midi".to_string(), kind: TokenKind::Namespace },
                Token { name: "channels".to_string(), kind: TokenKind::ArrayIndex(3) },
                Token { name: "controls".to_string(), kind: TokenKind::ArrayIndex(15) },
                Token { name: "value".to_string(), kind: TokenKind::Leaf }
            ]
        );

        // Checking if a note is currently pressed
        assert_eq!(
            Parser::new("state.midi.note_on(*, 7)").parse().unwrap(),
            vec![
                Token { name: "state".to_string(), kind: TokenKind::Namespace },
                Token { name: "midi".to_string(), kind: TokenKind::Namespace },
                func_token("note_on", vec!["*", "7"])
            ]
        );
    }

    // Shorthand helper function to make assertions much less wordy
    fn func_token(name: &str, args: Vec<&str>) -> Token {
        Token {
            name: name.to_string(),
            kind: TokenKind::FunctionCall(
                args
                    .iter()
                    .map(|s| s.to_string())
                    .collect()
            )
        }
    }
}