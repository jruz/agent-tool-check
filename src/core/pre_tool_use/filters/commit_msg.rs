use brush_parser::{ast, word};
use regex::Regex;

use super::shell;

const COMMIT_TITLE_PATTERN: &str = r"^\[(feat|bugs|refa|deps|tool)\] \S+.*: .+";
const SHORT_VALUE_FLAGS: [char; 5] = ['m', 'F', 'C', 'c', 't'];

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Never include Co-Authored-By in commit messages")]
    Coauthor,
    #[error("Invalid commit title, follow the Skill(commit) format")]
    InvalidTitle,
    #[error("Could not extract commit title")]
    MissingTitle,
    #[error(transparent)]
    Regex(#[from] regex::Error),
    #[error(transparent)]
    Shell(#[from] shell::Error),
}

enum CommitOption {
    Message(Option<String>),
    MessageFromStdin,
    Other,
    Skip,
}

type Parsed<'arguments> = (CommitOption, &'arguments [Option<String>]);

pub fn run(arguments: &[&ast::Word], stdin: Option<&str>) -> Result<(), Error> {
    let values = arguments
        .iter()
        .map(|word| message_text(word))
        .collect::<Result<Vec<_>, _>>()?;
    if values
        .iter()
        .flatten()
        .map(String::as_str)
        .chain(stdin)
        .any(|value| value.contains("Co-Authored-By"))
    {
        return Err(Error::Coauthor);
    }
    let options = commit_options(&values);
    let skips_title = options
        .iter()
        .any(|option| matches!(option, CommitOption::Skip));
    if skips_title {
        Ok(())
    } else {
        check_title(&options, stdin)
    }
}

fn commit_options(arguments: &[Option<String>]) -> Vec<CommitOption> {
    let Some((argument, remaining)) = arguments.split_first() else {
        return Vec::new();
    };
    let (option, remaining) = match argument.as_deref() {
        Some(argument) => parse_option(argument, remaining),
        None => (CommitOption::Other, remaining),
    };
    let later_options = commit_options(remaining);
    std::iter::once(option).chain(later_options).collect()
}

fn parse_option<'arguments>(
    argument: &str,
    remaining: &'arguments [Option<String>],
) -> Parsed<'arguments> {
    match argument {
        "--no-edit" | "--fixup" => (CommitOption::Skip, remaining),
        "--message" => separate_message(remaining),
        "--file" => separate_file(remaining),
        "--trailer" | "--author" => separate_other(remaining),
        _ => attached_option(argument, remaining),
    }
}

fn separate_message(remaining: &[Option<String>]) -> Parsed<'_> {
    let (value, remaining) = next_value(remaining);
    (CommitOption::Message(value), remaining)
}

fn separate_file(remaining: &[Option<String>]) -> Parsed<'_> {
    let (value, remaining) = next_value(remaining);
    let path = value.as_deref();
    (file_message(path), remaining)
}

fn separate_other(remaining: &[Option<String>]) -> Parsed<'_> {
    let (_, remaining) = next_value(remaining);
    (CommitOption::Other, remaining)
}

fn attached_option<'arguments>(
    argument: &str,
    remaining: &'arguments [Option<String>],
) -> Parsed<'arguments> {
    if argument.starts_with("--fixup=") {
        return (CommitOption::Skip, remaining);
    }
    if let Some(message) = argument.strip_prefix("--message=") {
        let text = message.to_owned();
        let value = Some(text);
        return (CommitOption::Message(value), remaining);
    }
    if let Some(file) = argument.strip_prefix("--file=") {
        let path = Some(file);
        return (file_message(path), remaining);
    }
    if argument.starts_with("--") {
        return (CommitOption::Other, remaining);
    }
    match argument.strip_prefix('-') {
        Some(flags) if !flags.is_empty() => short_cluster(flags, remaining),
        _ => (CommitOption::Other, remaining),
    }
}

fn short_cluster<'arguments>(
    flags: &str,
    remaining: &'arguments [Option<String>],
) -> Parsed<'arguments> {
    let mut characters = flags.chars();
    let Some(flag) = characters.find(|flag| SHORT_VALUE_FLAGS.contains(flag)) else {
        return (CommitOption::Other, remaining);
    };
    let attached = characters.as_str();
    let (value, remaining) = short_value(attached, remaining);
    (short_option(flag, value), remaining)
}

fn short_value<'arguments>(
    attached: &str,
    remaining: &'arguments [Option<String>],
) -> (Option<String>, &'arguments [Option<String>]) {
    if attached.is_empty() {
        return next_value(remaining);
    }
    let text = attached.to_owned();
    (Some(text), remaining)
}

fn short_option(flag: char, value: Option<String>) -> CommitOption {
    match flag {
        'm' => CommitOption::Message(value),
        'F' => {
            let path = value.as_deref();
            file_message(path)
        }
        _ => CommitOption::Other,
    }
}

fn next_value(remaining: &[Option<String>]) -> (Option<String>, &[Option<String>]) {
    match remaining.split_first() {
        Some((value, remaining)) => (value.clone(), remaining),
        None => (None, remaining),
    }
}

fn file_message(path: Option<&str>) -> CommitOption {
    match path {
        Some("-") => CommitOption::MessageFromStdin,
        _ => CommitOption::Message(None),
    }
}

fn check_title(options: &[CommitOption], stdin: Option<&str>) -> Result<(), Error> {
    let title = extract_title(options, stdin).ok_or(Error::MissingTitle)?;
    let expression = Regex::new(COMMIT_TITLE_PATTERN)?;
    if expression.is_match(title) {
        Ok(())
    } else {
        Err(Error::InvalidTitle)
    }
}

fn extract_title<'message>(
    options: &'message [CommitOption],
    stdin: Option<&'message str>,
) -> Option<&'message str> {
    let source = options
        .iter()
        .find(|option| !matches!(option, CommitOption::Other))?;
    match source {
        CommitOption::Message(message) => message.as_deref()?.lines().next(),
        CommitOption::MessageFromStdin => stdin?.lines().next(),
        CommitOption::Other | CommitOption::Skip => None,
    }
}

fn message_text(word: &ast::Word) -> Result<Option<String>, Error> {
    if let Some(text) = shell::static_word(word)? {
        let text = Some(text);
        return Ok(text);
    }
    let pieces = shell::parse_word(&word.value)?;
    heredoc_word(&pieces)
}

fn heredoc_word(pieces: &[word::WordPieceWithSource]) -> Result<Option<String>, Error> {
    let texts = pieces
        .iter()
        .map(heredoc_piece)
        .collect::<Result<Option<Vec<_>>, _>>()?;
    let text = texts.map(|parts| parts.concat());
    Ok(text)
}

fn heredoc_piece(piece: &word::WordPieceWithSource) -> Result<Option<String>, Error> {
    match &piece.piece {
        word::WordPiece::Text(text) | word::WordPiece::SingleQuotedText(text) => {
            let owned = text.clone();
            let value = Some(owned);
            Ok(value)
        }
        word::WordPiece::DoubleQuotedSequence(nested) => heredoc_word(nested),
        word::WordPiece::CommandSubstitution(source) => heredoc_source(source),
        word::WordPiece::AnsiCQuotedText(_)
        | word::WordPiece::GettextDoubleQuotedSequence(_)
        | word::WordPiece::TildeExpansion(_)
        | word::WordPiece::ParameterExpansion(_)
        | word::WordPiece::BackquotedCommandSubstitution(_)
        | word::WordPiece::EscapeSequence(_)
        | word::WordPiece::ArithmeticExpression(_) => Ok(None),
    }
}

fn heredoc_source(source: &str) -> Result<Option<String>, Error> {
    let program = shell::parse_program(source)?;
    let [commands] = program.complete_commands.as_slice() else {
        return Ok(None);
    };
    let [item] = commands.0.as_slice() else {
        return Ok(None);
    };
    let pipeline = &item.0.first;
    if !item.0.additional.is_empty()
        || pipeline.bang
        || matches!(item.1, ast::SeparatorOperator::Async)
    {
        return Ok(None);
    }
    match pipeline.seq.as_slice() {
        [ast::Command::Simple(command)] => heredoc_command(command),
        _ => Ok(None),
    }
}

fn heredoc_command(command: &ast::SimpleCommand) -> Result<Option<String>, Error> {
    let Some(word) = command.word_or_name.as_ref() else {
        return Ok(None);
    };
    let executable = shell::static_word(word)?;
    if executable.as_deref() != Some("cat") {
        return Ok(None);
    }
    let suffix = command.suffix.iter().flat_map(|suffix| &suffix.0);
    let items = command
        .prefix
        .iter()
        .flat_map(|prefix| &prefix.0)
        .chain(suffix)
        .collect::<Vec<_>>();
    match items.as_slice() {
        [ast::CommandPrefixOrSuffixItem::IoRedirect(redirect)] => heredoc_redirect(redirect),
        _ => Ok(None),
    }
}

fn heredoc_redirect(redirect: &ast::IoRedirect) -> Result<Option<String>, Error> {
    match redirect {
        ast::IoRedirect::HereDocument(_, document) => {
            let text = shell::heredoc_text(document)?;
            Ok(text)
        }
        ast::IoRedirect::File(_, _, _)
        | ast::IoRedirect::HereString(_, _)
        | ast::IoRedirect::OutputAndError(_, _) => Ok(None),
    }
}
