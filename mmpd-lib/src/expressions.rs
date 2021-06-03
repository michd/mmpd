mod functions;

use std::fmt::{self, Display, Formatter};

#[derive(PartialEq, Debug)]
pub struct Token {
    pub name: String,
    pub kind: TokenKind,
}

#[derive(PartialEq, Debug)]
pub enum TokenKind {
    Leaf,
    ArrayIndex(usize),
    Namespace,
    FunctionCall(Vec<String>)
}

#[derive(Debug)]
pub struct ExpressionError<'a> {
    message: String,
    var_str: &'a str,
    location: usize
}

impl <'a> ExpressionError<'a> {
    fn new(message: String, location: usize) -> ExpressionError<'a> {
        ExpressionError {
            message: message.to_string(),
            var_str: "",
            location
        }
    }

    fn offset_location(mut self, location_offset: usize) -> Self {
        self.location += location_offset;
        self
    }

    fn add_var_str(mut self, var_str: &'a str) -> Self {
        self.var_str = var_str;
        self
    }
}

impl Display for ExpressionError<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Variable Parsing Error: {}\n\nVariable notation:\n    {}\n{}___^",
            self.message,
            self.var_str,
            " ".repeat(self.location + 1)
        )
    }
}

fn is_space(c: &char) -> bool {
    [
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
    ].contains(c)
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

// Result of a function reading a number of characters
struct ReadResult {
    // If this function read name characters, they are in here
    name: Option<String>,

    // If this function resulted in finishing reading an entire token, this is the token kind
    token_kind: Option<TokenKind>,

    // The state of the parser after this function, instructing it what to read next
    state: ParserState,

    // Number of characters this function has consumed
    chars_read: usize
}

fn parse(var_str: &str) -> Result<Vec<Token>, ExpressionError> {
    if var_str.is_empty() {
        return Err(ExpressionError::new("Empty variable notation string".to_string(), 0));
    }

    let mut current_name: Option<String> = None;
    let mut index: usize = 0;
    let mut state = ParserState::Name;
    let mut tokens: Vec<Token> = vec![];

    loop {
        let read_result = match state {
            ParserState::Name => read_name_chars(&var_str[index..]),
            ParserState::Array => read_array_chars(&var_str[index..]),
            ParserState::FunctionCall => functions::read_function_call_chars(&var_str[index..]),
            ParserState::AfterToken => read_after_token_chars(&var_str[index..]),
            ParserState::End => break
        }.map_err(|err| {
            // Adjust received error location by index known here, and include var_str
            err
                .offset_location(index)
                .add_var_str(var_str)
        })?;

        current_name = current_name.or(read_result.name);

        // If there is a finished token to consume, build it and add it to the list
        if let Some(token_kind) = read_result.token_kind {
            if current_name.is_none() {
                // Well that very much shouldn't happen.
                panic!("Got a read_result with a token_kind while we have no current_name.");
            }

            tokens.push(Token {
                name: current_name.unwrap(),
                kind: token_kind
            });

            current_name = None;
        }

        state = read_result.state;
        index += read_result.chars_read;

        if index >=var_str.len() {
            break;
        }
    }

    Ok(tokens)
}

fn read_name_chars(var_str: &str) -> Result<ReadResult, ExpressionError> {
    let mut name = "".to_string();
    let mut space_seen = false;

    for (i, c) in var_str.chars().enumerate() {
        match c {
            // Valid token name characters
            'a'..='z' | 'A'..='Z' | '0'..='9' | '_' => {
                if !name.is_empty() && space_seen {
                    return Err(ExpressionError::new(
                        "Space characters within names are not supported.".to_string(),
                        i
                    ));
                }

                name.push_str(c.to_string().as_str())
            }

            '.' => {
                return Ok(ReadResult {
                    name: Some(name),
                    token_kind: Some(TokenKind::Namespace),
                    state: ParserState::Name,
                    chars_read: i + 1
                });
            }

            '[' if name.is_empty() => return Err(ExpressionError::new(
                "Missing name for array access".to_string(),
                i
            )),

            '[' if space_seen => return Err(ExpressionError::new(
                "Unexpected space before array index access".to_string(),
                i - 1
            )),

            '[' => {
                return Ok(ReadResult {
                    name: Some(name),
                    token_kind: None,
                    state: ParserState::Array,
                    chars_read: i + 1
                });
            },

            '(' => {
                return Ok(ReadResult {
                    name: Some(name),
                    token_kind: None,
                    state: ParserState::FunctionCall,
                    chars_read: i + 1
                })
            },

            _ if is_space(&c) && !name.is_empty() => space_seen = true,

            _ if is_space(&c) => {
                // Spaces preceding a name are fine.
            }

            _ => {
                return Err(ExpressionError::new(
                    format!("Invalid character '{}'", c.to_string()),
                    1
                ));
            }
        }
    }

    if name.is_empty() {
        Err(ExpressionError::new(
            "Unexpected end of variable notation string; expecting a name.".to_string(),
            0
        ))
    } else {
        Ok(ReadResult {
            name: Some(name),
            token_kind: Some(TokenKind::Leaf),
            state: ParserState::End,
            chars_read: var_str.len()
        })
    }
}

fn read_array_chars(var_str: &str) -> Result<ReadResult, ExpressionError> {
    let mut arr_index_str = "".to_string();

    let mut space_seen = false;

    for (i, c) in var_str.chars().enumerate() {
        match c {
            '0'..='9' if space_seen => return Err(ExpressionError::new(
                "Unexpected space in array index".to_string(),
                i - 1
            )),

            '0'..='9' => {
                arr_index_str.push_str(c.to_string().as_str());
            }

            ']' => {
                if arr_index_str.is_empty() {
                    return Err(ExpressionError::new("Missing array index".to_string(), 0));
                }

                return match usize::from_str_radix(arr_index_str.as_str(), 10) {
                    Ok(arr_index) => {
                        return Ok(ReadResult {
                            name: None,
                            token_kind: Some(TokenKind::ArrayIndex(arr_index)),
                            state: ParserState::AfterToken,
                            chars_read: i + 1
                        });
                    }

                    Err(e) => {
                        Err(ExpressionError::new(
                            format!(
                                "Failed to parse array index '{}': {}",
                                arr_index_str,
                                e
                            ),
                            i - 1
                        ))
                    }
                }
            }

            _ if !arr_index_str.is_empty() && is_space(&c) => space_seen = true,

            _ if is_space(&c) => {
                // Leading spaces are fine
            }

            _ => {
                return Err(ExpressionError::new(
                    format!("Invalid character '{}', expecting decimal digit or ']'.", c),
                    i
                ));
            }
        }
    }

    Err(ExpressionError::new(
        "Unexpected end of variable notation string; expecting decimal digits or ']'".to_string(),
        var_str.len()
    ))
}

fn read_after_token_chars(var_str: &str) -> Result<ReadResult, ExpressionError> {
    for (i, c) in var_str.chars().enumerate() {
        match c {
            '.' => return Ok(ReadResult {
                name: None,
                token_kind: None,
                state: ParserState::Name,
                chars_read: 1
            }),

            _ if is_space(&c) => {
                // Spaces are fine, ignore.
            }

            _ => return Err(ExpressionError::new(
                format!("Invalid character '{}'; expected '.', spaces, or end.", c),
                i
            ))
        }
    }

    Ok(ReadResult {
        name: None,
        token_kind: None,
        state: ParserState::End,
        chars_read: 0
    })
}

#[cfg(test)]
mod tests {
    use crate::expressions::{parse, Token, TokenKind};

    #[test]
    fn parses_a_single_leaf_node() {
        assert_eq!(
            parse("leaf_token").unwrap(),
            vec![Token { name: "leaf_token".to_string(), kind: TokenKind::Leaf }]
        );

        assert!(
            parse("!!invalid name!!").is_err()
        )
    }

    #[test]
    fn parses_a_leaf_after_a_namespace() {
        assert_eq!(
            parse("my_namespace.my_leaf").unwrap(),
            vec![
                Token { name: "my_namespace".to_string(), kind: TokenKind::Namespace },
                Token { name: "my_leaf".to_string(), kind: TokenKind::Leaf }
            ]
        );

        // Wrong separator
        assert!(
            parse("my_namespace|my_leaf").is_err()
        );
    }

    #[test]
    fn parses_multiple_namespaces() {
        assert_eq!(
            parse("my_namespace.my_sub_namespace.my_leaf").unwrap(),
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
            parse("my_namespace.arr[823]").unwrap(),
            vec![
                Token { name: "my_namespace".to_string(), kind: TokenKind::Namespace },
                Token { name: "arr".to_string(), kind: TokenKind::ArrayIndex(823) }
            ]
        );

        // With spaces within square brackets
        assert_eq!(
            parse("arr[ 24 ]").unwrap(),
            vec![Token { name: "arr".to_string(), kind: TokenKind::ArrayIndex(24) }]
        );

        // Missing name preceding array index
        assert!(
            parse("[23]").is_err()
        );

        // Unclosed array notation
        assert!(
            parse("my_namespace.arr[823").is_err()
        );

        // Non-digits in index
        assert!(
            parse("my_namespace.arr[INVALID]").is_err()
        );

        // Spaces between characters
        assert!(
            parse("ns.arr[34 2 4]").is_err()
        )
    }

    #[test]
    fn allows_spaces_between_tokens() {
        assert_eq!(
            parse("ns    .  leaf").unwrap(),
            vec![
                Token { name: "ns".to_string(), kind: TokenKind::Namespace },
                Token { name: "leaf".to_string(), kind: TokenKind::Leaf }
            ]
        );
    }

    #[test]
    fn disallows_spaces_within_token() {
        assert!(
            parse("namespace.something with a space").is_err()
        );
    }

    #[test]
    fn disallows_spaces_before_array_bracket() {
        assert!(
            parse("arr [323]").is_err()
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
            parse("my_namespace.func(abc, 123, sup)").unwrap(),
            vec![
                Token { name: "my_namespace".to_string(), kind: TokenKind::Namespace },
                func_token("func", vec!["abc", "123", "sup"])
            ]
        );

        assert_eq!(
            parse(r#"fun("ab")"#).unwrap(),
            vec![func_token("fun", vec!["ab"])]
        );

        assert_eq!(
            parse(r#"fun("abc",  foo   )"#).unwrap(),
            vec![func_token("fun", vec!["abc", "foo"])]
        );

        assert_eq!(
            parse(r#"fun(arg1,"arg2",arg3)"#).unwrap(),
            vec![func_token("fun", vec!["arg1", "arg2", "arg3"])]
        );

        assert_eq!(
            parse(r#"fun(arg1, "arg number two", "arg \"three\"")"#).unwrap(),
            vec![func_token("fun", vec!["arg1", "arg number two", r#"arg "three""#])]
        );

        assert_eq!(
            parse(r#"fun(ar\g1   ,"ar\g2",  arg3)"#).unwrap(),
            vec![func_token("fun", vec![r#"ar\g1"#, r#"arg2"#, "arg3"])]
        );

        assert_eq!(
            parse(r#"fun(ar\\g1, "arg\\2", arg3)"#).unwrap(),
            vec![func_token("fun", vec![r#"ar\\g1"#, r#"arg\2"#, "arg3"])]
        );

        assert_eq!(
            parse("fun_1_2()").unwrap(),
            vec![func_token("fun_1_2", vec![])]
        );

        assert_eq!(
            parse("fun(        )").unwrap(),
            vec![func_token("fun", vec![])]
        );

        assert_eq!(
            parse("fun(  spaces_before_arg)").unwrap(),
            vec![func_token("fun", vec!["spaces_before_arg"])]
        );

        assert_eq!(
            parse("fun(spaces_after_arg   )").unwrap(),
            vec![func_token("fun", vec!["spaces_after_arg"])]
        );

        assert_eq!(
            parse(r#"fun(*, C3)"#).unwrap(),
            vec![func_token("fun", vec!["*", "C3"])]
        );

        assert_eq!(
            parse(
                r#"namespace.fun1(f1_arg1, "f1 arg \\two\\").fun2("f2 \"arg\" one", f2_arg_two)"#
            ).unwrap(),
            vec![
                Token { name: "namespace".to_string(), kind: TokenKind::Namespace },
                func_token("fun1", vec!["f1_arg1", r#"f1 arg \two\"#]),
                func_token("fun2", vec![r#"f2 "arg" one"#, "f2_arg_two"])
            ]
        );

        // Invalid characters
        assert!(
            parse("my_namespace.func(abc), 123)").is_err()
        );

        // Unclosed parentheses
        assert!(
            parse("fun(foo").is_err()
        );

        // Unclosed quotes, which means it's interpreted as missing quote /
        // missing closing parentheses
        assert!(
            parse(r#"fun("foo)"#).is_err()
        );

        // Uncloses parentheses, the opening paren at the end is irrelevant
        assert!(
            parse(r#"fun("foo", bar("#).is_err()
        );

        // Unclosed parenthesis due to still being in quoted string due to escaped quote
        assert!(
            parse(r#"fun("arg\")"#).is_err()
        );

        // Unexpected comma
        assert!(
            parse("fun(,foo)").is_err()
        );

        // Unexpected character after closing quotes for an arg; expects a comma
        assert!(
            parse(r#"fun("abc" asdf"#).is_err()
        );

        // Spaces in unquoted arguments aren't supported
        assert!(
            parse("fun(  bare value should not have spaces in middle , foo)").is_err()
        )
    }

    #[test]
    fn parses_realistic_examples() {
        // Accessing a field from the event that triggered this action
        assert_eq!(
            parse("event.key").unwrap(),
            vec![
                Token { name: "event".to_string(), kind: TokenKind::Namespace },
                Token { name: "key".to_string(), kind: TokenKind::Leaf },
            ]
        );

        // Accessing a specific MIDI control value from state
        assert_eq!(
            parse("state.midi.channels[3].controls[15].value").unwrap(),
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
            parse("state.midi.note_on(*, 7)").unwrap(),
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