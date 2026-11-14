use brush_parser::{ast, word};

use super::super::shell;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Never print environment variables")]
    Dump,
    #[error("Shell option requires a value")]
    MissingOptionValue,
    #[error(
        "Never print variables with TOKEN, KEY, SECRET, PASSWORD, or PASSPHRASE name parts (case-insensitive). Unsupported output expansions remain blocked."
    )]
    RestrictedOutput,
    #[error(transparent)]
    Shell(#[from] shell::Error),
}

pub fn run(executable: &str, arguments: &[&ast::Word]) -> Result<(), Error> {
    match executable {
        "printenv" => check_printenv(arguments),
        "echo" | "printf" | "print" => arguments
            .iter()
            .try_for_each(|word| check_output_word(word)),
        "set" if arguments.is_empty() => Err(Error::Dump),
        "set" | "bash" | "dash" | "ksh" | "sh" | "zsh" => check_trace_arguments(arguments),
        "declare" | "export" | "local" | "readonly" | "typeset" => check_binding_output(arguments),
        "ps" => check_process_output(arguments),
        _ => Ok(()),
    }
}

pub fn check_output(output: shell::Output<'_>) -> Result<(), Error> {
    match output {
        shell::Output::Environment => Err(Error::Dump),
        shell::Output::Expansion(pieces) => check_output_expansion(pieces),
    }
}

fn check_output_expansion(pieces: &[word::WordPieceWithSource]) -> Result<(), Error> {
    pieces
        .iter()
        .try_for_each(|piece| check_output_piece(&piece.piece))
}

fn check_output_word(word: &ast::Word) -> Result<(), Error> {
    let pieces = shell::parse_word(&word.value)?;
    let output = shell::Output::Expansion(&pieces);
    check_output(output)
}

fn check_output_piece(piece: &word::WordPiece) -> Result<(), Error> {
    match piece {
        word::WordPiece::ParameterExpansion(expression) => check_output_parameter(expression),
        word::WordPiece::ArithmeticExpression(_) | word::WordPiece::TildeExpansion(_) => {
            Err(Error::RestrictedOutput)
        }
        word::WordPiece::DoubleQuotedSequence(pieces)
        | word::WordPiece::GettextDoubleQuotedSequence(pieces) => check_output_expansion(pieces),
        word::WordPiece::Text(_)
        | word::WordPiece::SingleQuotedText(_)
        | word::WordPiece::AnsiCQuotedText(_)
        | word::WordPiece::EscapeSequence(_)
        | word::WordPiece::CommandSubstitution(_)
        | word::WordPiece::BackquotedCommandSubstitution(_) => Ok(()),
    }
}

fn check_output_parameter(expression: &word::ParameterExpr) -> Result<(), Error> {
    match shell::expression_parameter(expression) {
        Some(word::Parameter::Named(name) | word::Parameter::NamedWithAllIndices { name, .. }) => {
            check_output_name(name)?;
        }
        Some(word::Parameter::NamedWithIndex { name, index }) => {
            check_output_name(name)?;
            check_output_value(index)?;
        }
        Some(word::Parameter::Special(word::SpecialParameter::LastExitStatus)) => {}
        Some(word::Parameter::Positional(_) | word::Parameter::Special(_)) | None => {
            return Err(Error::RestrictedOutput);
        }
    }
    shell::parameter_words(expression)
        .into_iter()
        .try_for_each(check_output_value)
}

fn check_output_value(value: &str) -> Result<(), Error> {
    let pieces = shell::parse_word(value)?;
    check_output_expansion(&pieces)
}

fn is_sensitive_name(name: &str) -> bool {
    name.split('_').any(|part| {
        ["TOKEN", "KEY", "SECRET", "PASSWORD", "PASSPHRASE"]
            .iter()
            .any(|sensitive| part.eq_ignore_ascii_case(sensitive))
    })
}

fn check_output_name(name: &str) -> Result<(), Error> {
    if is_sensitive_name(name) {
        return Err(Error::RestrictedOutput);
    }
    Ok(())
}

fn check_printenv(arguments: &[&ast::Word]) -> Result<(), Error> {
    let literals = arguments
        .iter()
        .map(|word| shell::static_word(word))
        .collect::<Result<Vec<_>, _>>()?;
    let mut names = literals
        .iter()
        .skip_while(|literal| matches!(literal.as_deref(), Some("-0" | "--null" | "--")));
    let first = names.next().ok_or(Error::Dump)?;
    std::iter::once(first)
        .chain(names)
        .try_for_each(|literal| match literal.as_deref() {
            Some(name) if !name.starts_with('-') && !is_sensitive_name(name) => Ok(()),
            Some(_) | None => Err(Error::Dump),
        })
}

fn check_binding_output(arguments: &[&ast::Word]) -> Result<(), Error> {
    arguments
        .iter()
        .try_for_each(|word| check_binding_word(word))?;
    let literals = arguments
        .iter()
        .map(|word| shell::static_word(word))
        .collect::<Result<Vec<_>, _>>()?;
    let only_options = literals.iter().all(|literal| {
        literal
            .as_deref()
            .is_some_and(|value| value.starts_with('-'))
    });
    let print_option = literals.iter().any(|literal| {
        literal
            .as_deref()
            .is_some_and(|value| value.starts_with('-') && value.contains('p'))
    });
    if only_options || print_option {
        return Err(Error::Dump);
    }
    Ok(())
}

fn check_binding_word(word: &ast::Word) -> Result<(), Error> {
    let literal = shell::static_word(word)?;
    if literal.is_none() && !shell::is_env_assignment(word) {
        return Err(Error::Dump);
    }
    Ok(())
}

fn check_process_output(arguments: &[&ast::Word]) -> Result<(), Error> {
    match arguments.split_first() {
        Some((word, remaining)) => check_process_argument(word, remaining),
        None => Ok(()),
    }
}

fn check_process_argument(word: &ast::Word, remaining: &[&ast::Word]) -> Result<(), Error> {
    let literal = shell::static_word(word)?;
    match literal.as_deref() {
        Some(
            "-p" | "--pid" | "-u" | "--user" | "-U" | "--User" | "-o" | "--format" | "--sort"
            | "-g" | "--group" | "-G" | "--Group" | "-t" | "--tty" | "-C",
        ) => {
            let remaining = remaining_after_option(remaining)?;
            check_process_output(remaining)
        }
        Some(value) if process_option_needs_value(value) => {
            let remaining = remaining_after_option(remaining)?;
            check_process_output(remaining)
        }
        Some(value) if !value.starts_with('-') && value.contains('e') => Err(Error::Dump),
        Some(_) => check_process_output(remaining),
        None => Err(Error::Dump),
    }
}

fn process_option_needs_value(value: &str) -> bool {
    let Some(options) = value
        .strip_prefix('-')
        .filter(|value| !value.starts_with('-'))
    else {
        return false;
    };
    let mut characters = options.chars();
    characters.any(|option| matches!(option, 'p' | 'u' | 'U' | 'o' | 'g' | 'G' | 't' | 'C'))
        && characters.next().is_none()
}

fn check_trace_arguments(arguments: &[&ast::Word]) -> Result<(), Error> {
    match arguments.split_first() {
        Some((word, remaining)) => check_trace_argument(word, remaining),
        None => Ok(()),
    }
}

fn check_trace_argument(word: &ast::Word, remaining: &[&ast::Word]) -> Result<(), Error> {
    let literal = shell::static_word(word)?;
    match literal.as_deref() {
        Some("-o") => check_trace_option(remaining),
        Some(value)
            if value.starts_with('-') && !value.starts_with("--") && value.contains('x') =>
        {
            Err(Error::Dump)
        }
        Some("-c" | "--command") => Ok(()),
        Some(value)
            if value.starts_with('-') && !value.starts_with("--") && value.contains('c') =>
        {
            Ok(())
        }
        Some(_) | None => check_trace_arguments(remaining),
    }
}

fn check_trace_option(arguments: &[&ast::Word]) -> Result<(), Error> {
    match arguments.split_first() {
        Some((word, remaining)) => {
            let literal = shell::static_word(word)?;
            if literal.as_deref() == Some("xtrace") {
                return Err(Error::Dump);
            }
            check_trace_arguments(remaining)
        }
        None => Ok(()),
    }
}

fn remaining_after_option<'words>(
    arguments: &'words [&ast::Word],
) -> Result<&'words [&'words ast::Word], Error> {
    let (_, remaining) = arguments.split_first().ok_or(Error::MissingOptionValue)?;
    Ok(remaining)
}
