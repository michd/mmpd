use super::{ReadResult, VariableError, TokenKind, ParserState, is_space};

#[derive(Debug)]
enum FunctionParseState {
    Start,
    BareValue,
    QuotedValue,
    AfterQuotedValue,
    End,
}

#[derive(Debug)]
struct FunctionReadResult {
    value: Option<String>,
    state: FunctionParseState,
    chars_read: usize,
}

pub(super) fn read_function_call_chars(var_str: &str) -> Result<ReadResult, VariableError> {
    let mut values: Vec<String> = vec![];
    let mut index: usize = 0;
    let mut state = FunctionParseState::Start;

    loop {
        let read_result = match state {
            FunctionParseState::Start => read_start(&var_str[index..]),
            FunctionParseState::BareValue => read_bare_value(&var_str[index..]),
            FunctionParseState::QuotedValue => read_quoted_value(&var_str[index..]),

            FunctionParseState::AfterQuotedValue => read_after_quoted_value(
                &var_str[index..]
            ),

            FunctionParseState::End => break
        }.map_err(|err| {
            err.offset_location(index)
        })?;

        if let Some(value) = read_result.value {
            values.push(value.to_owned())
        }

        state = read_result.state;

        // Point index at next character to start reading
        index += read_result.chars_read;

        // If we've reached the end this might point beyond the end, so check first.
        if index >= var_str.len() {
            break;
        }
    }

    Ok(ReadResult {
        name: None,
        token_kind: Some(TokenKind::FunctionCall(values)),
        state: ParserState::AfterToken,
        chars_read: index
    })
}

fn read_start(var_str: &str) -> Result<FunctionReadResult, VariableError> {
    for (i, c) in var_str.chars().enumerate() {
        match c {
            '"' => return Ok(FunctionReadResult {
                value: None,
                state: FunctionParseState::QuotedValue,
                chars_read: i + 1
            }),

            ',' => return Err(VariableError::new(
                "Unexpected ',' in function invocation syntax".to_string(),
                0
            )),

            ')' => return Ok(FunctionReadResult {
                value: None,
                state: FunctionParseState::End,
                chars_read: i + 1
            }),

            // TODO? Check for '%' and error on that?

            _ if is_space(&c) => {
                // Ignore whitespaces at start
            }

            _ => return Ok(FunctionReadResult {
                value: None,
                state: FunctionParseState::BareValue,

                // Note we're missing `+ 1` here, which allows the bare value reading function to
                // start from this offset and consume the character instead of consuming it here
                chars_read: i
            })
        }
    }

    Err(VariableError::new(
        "Unexpected end of function invocation syntax".to_string(),
        0
    ))
}

fn read_bare_value(var_str: &str) -> Result<FunctionReadResult, VariableError> {
    let mut value = "".to_string();

    let mut spaces_seen = false;

    for (i, c) in var_str.chars().enumerate() {
        match c {
            '"' => return Err(VariableError::new(
                "Unexpected '\"' quote character mid-value".to_string(),
                i
            )),

            ',' if value.is_empty() => return Err(VariableError::new(
                "Unexpected ',' in function values".to_string(),
                i
            )),

            ',' if !value.is_empty() => return Ok(FunctionReadResult {
                value: Some(value),
                // Expecting the same possible values as right after opening '(', so setting state
                // back to Start
                state: FunctionParseState::Start,
                chars_read: i + 1
            }),

            // TODO: ensure '%' isn't valid inside a bare value?

            ')' => return Ok(FunctionReadResult {
                value: if value.is_empty() { None } else { Some(value) },
                state: FunctionParseState::End,
                chars_read: i + 1
            }),

            _ if is_space(&c) && !value.is_empty() => {
                // Seeing whitespace after valid value characters, mark it.
                // Any valid value characters are no longer valid after whitespace.
                spaces_seen = true;
            }

            _ if !value.is_empty() && spaces_seen => {
                return Err(VariableError::new(
                    "Bare values in function invocation may not contain spaces".to_string(),
                    i - 1
                ));
            }

            _ => value.push_str(c.to_string().as_str())
        }
    }

    Err(VariableError::new(
        "Unexpected end of variable notation string during function call syntax \
        while reading bare value".to_string(),
        var_str.len()
    ))
}

fn read_quoted_value(var_str: &str) -> Result<FunctionReadResult, VariableError> {
    let mut value = "".to_string();
    let mut is_escaping = false;

    for (i, c) in var_str.chars().enumerate() {
        match c {
            '"' if is_escaping => {
                value.push_str(c.to_string().as_str());
                is_escaping = false;
            }

            '"' if !is_escaping => {
                return Ok (FunctionReadResult {
                    value: Some(value),
                    state: FunctionParseState::AfterQuotedValue,
                    chars_read: i + 1
                });
            }

            '\\' if is_escaping => {
                value.push_str(c.to_string().as_str());
                is_escaping = false;
            }

            '\\' if !is_escaping => is_escaping = true,

            _ => {
                value.push_str(c.to_string().as_str());
                is_escaping = false;
            }
        }
    }

    Err(VariableError::new(
        "Unexpected end of variable notation string during function call syntax \
        while reading quoted value".to_string(),
        var_str.len()
    ))
}

fn read_after_quoted_value(var_str: &str) -> Result<FunctionReadResult, VariableError> {
    for (i, c) in var_str.chars().enumerate() {
        match c {
            ',' => return Ok(FunctionReadResult {
                value: None,
                // A comma after a known value leaves us in the same state after the opening '('
                // that started the function: expecting a new value to begin, or ')'. As such,
                // the state is set back to Start.
                state: FunctionParseState::Start,
                chars_read: i + 1
            }),

            ')' => return Ok(FunctionReadResult {
                value: None,
                state: FunctionParseState::End,
                chars_read: i + 1
            }),

            _ if is_space(&c) => {
                // Carry on, looking for relevant character.
            }

            _ => return Err(VariableError::new(
                format!(
                    "Unexpected character '{}' in function call syntax within variable notation \
                    string, expecting ',', ')', or whitespace.",
                    c
                ),
                i
            ))
        }
    }

    Err(VariableError::new(
        "Unexpected end of string in function call syntax within \
        variable notation.".to_string(),
        var_str.len()
    ))
}