use std::num::{ParseFloatError, ParseIntError};

use crate::lexical_token::{
    FloatValue, IntValue, LexicalToken, Name, Punctuator, PunctuatorType, StringValue,
};
use crate::scanner::{ScanError, Scanner};
use crate::Span;
use logos::{Lexer, Logos};

mod block_string_scanner;
mod string_scanner;

#[derive(Logos, Debug, PartialEq)]
#[logos(subpattern intpart = r"-?(?:0|[1-9]\d*)")]
#[logos(subpattern decimalpart = r"\.\d+")]
#[logos(subpattern exponentpart = r"[eE][+-]?\d+")]
#[logos(subpattern hexdigit = r"[0-9A-Fa-f]")]
#[logos(subpattern fixedunicode = r"\\u(?&hexdigit)(?&hexdigit)(?&hexdigit)(?&hexdigit)")]
pub enum Token<'a> {
    // Punctuators
    #[token("!")]
    Bang,
    #[token("$")]
    Dollar,
    #[token("&")]
    Ampersand,
    #[token("(")]
    OpenRoundBracket,
    #[token(")")]
    CloseRoundBracket,
    #[token("...")]
    Ellipse,
    #[token(":")]
    Colon,
    #[token("=")]
    Equals,
    #[token("@")]
    At,
    #[token("[")]
    OpenSquareBracket,
    #[token("]")]
    CloseSquareBracket,
    #[token("{")]
    OpenBrace,
    #[token("}")]
    CloseBrace,
    #[token("|")]
    Pipe,

    // Name
    #[regex(r"[a-zA-Z_]\w*")]
    Name(&'a str),

    // IntValue
    #[regex(r"(?&intpart)", parse_integer)]
    IntValue(Result<i32, ParseIntError>),

    // FloatValue
    #[regex(
        r"(?&intpart)(?:(?&decimalpart)(?&exponentpart)|(?&decimalpart)|(?&exponentpart))",
        parse_float
    )]
    FloatValue(Result<f64, ParseFloatError>),

    // StringValue
    #[regex(r#""(?:[^\\"\n\r]|(?&fixedunicode)|\\u\{(?&hexdigit)+\}|\\["\\/bfnrt])*""#r, parse_string)]
    StringValue(Result<String, Vec<Span>>),

    #[regex("\"\"\"", parse_block_string)]
    BlockStringValue(String),

    // Skippable
    #[error]
    #[regex(r"[\uFEFF\t \n\r,]+", logos::skip)]
    #[regex(r"#[^\n\r]*", logos::skip)] // comments
    Error,
}

fn parse_block_string<'a>(lexer: &mut Lexer<'a, Token<'a>>) -> Option<String> {
    match block_string_scanner::Token::parse(lexer.remainder()) {
        Ok((s, bytes_consumed)) => {
            lexer.bump(bytes_consumed);
            Some(s)
        }
        Err(_) => {
            lexer.bump(lexer.remainder().len());
            None
        }
    }
}

fn parse_string<'a>(lexer: &mut Lexer<'a, Token<'a>>) -> Result<String, Vec<Span>> {
    string_scanner::Token::parse(lexer.slice(), lexer.span().start)
}

fn validate_number<'a>(lexer: &mut Lexer<'a, Token<'a>>) -> bool {
    let invalid_trail_bytes = lexer
        .remainder()
        .chars()
        .position(|c| !(c.is_ascii_alphanumeric() || matches!(c, '_' | '.')))
        .unwrap_or(lexer.remainder().len());

    lexer.bump(invalid_trail_bytes);

    invalid_trail_bytes == 0
}

fn parse_integer<'a>(lexer: &mut Lexer<'a, Token<'a>>) -> Option<Result<i32, ParseIntError>> {
    validate_number(lexer).then(|| lexer.slice().parse())
}

fn parse_float<'a>(lexer: &mut Lexer<'a, Token<'a>>) -> Option<Result<f64, ParseFloatError>> {
    validate_number(lexer).then(|| lexer.slice().parse())
}

#[repr(transparent)]
pub struct LogosScanner<'a>(Lexer<'a, Token<'a>>);

impl<'a> Iterator for LogosScanner<'a> {
    type Item = Result<LexicalToken<'a>, ScanError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|token| {
            let span = Span::new(self.0.span());
            match token {
                Token::Bang => punctuator(PunctuatorType::Bang, span),
                Token::Dollar => punctuator(PunctuatorType::Dollar, span),
                Token::Ampersand => punctuator(PunctuatorType::Ampersand, span),
                Token::OpenRoundBracket => punctuator(PunctuatorType::OpenRoundBracket, span),
                Token::CloseRoundBracket => punctuator(PunctuatorType::CloseRoundBracket, span),
                Token::Ellipse => punctuator(PunctuatorType::Ellipse, span),
                Token::Colon => punctuator(PunctuatorType::Colon, span),
                Token::Equals => punctuator(PunctuatorType::Equals, span),
                Token::At => punctuator(PunctuatorType::At, span),
                Token::OpenSquareBracket => punctuator(PunctuatorType::OpenSquareBracket, span),
                Token::CloseSquareBracket => punctuator(PunctuatorType::CloseSquareBracket, span),
                Token::OpenBrace => punctuator(PunctuatorType::OpenBrace, span),
                Token::CloseBrace => punctuator(PunctuatorType::CloseBrace, span),
                Token::Pipe => punctuator(PunctuatorType::Pipe, span),
                Token::Name(s) => Ok(LexicalToken::Name(Name::new(s, span))),
                Token::IntValue(res) => match res {
                    Ok(val) => Ok(LexicalToken::IntValue(IntValue::new(val, span))),
                    Err(_) => Err(ScanError::IntegerValueTooLarge(span)),
                },
                Token::FloatValue(res) => match res {
                    Ok(val) => Ok(LexicalToken::FloatValue(FloatValue::new(val, span))),
                    Err(_) => Err(ScanError::FloatValueTooLarge(span)),
                },
                Token::StringValue(res) => res
                    .map(|s| LexicalToken::StringValue(StringValue::new(s, span)))
                    .map_err(ScanError::StringWithInvalidEscapedUnicode),
                Token::BlockStringValue(s) => {
                    Ok(LexicalToken::StringValue(StringValue::new(s, span)))
                }
                Token::Error => Err(ScanError::UnrecognizedTokenError(span)),
            }
        })
    }
}

fn punctuator<'a>(pt: PunctuatorType, span: Span) -> Result<LexicalToken<'a>, ScanError> {
    Ok(LexicalToken::Punctuator(Punctuator::new(pt, span)))
}

impl<'a> Scanner<'a> for LogosScanner<'a> {
    fn empty_span(&self) -> Span {
        let n = self.0.span().start;
        Span::new(n..n)
    }
}

impl<'a> LogosScanner<'a> {
    pub fn new(s: &'a <Token<'a> as Logos<'a>>::Source) -> Self {
        Self(Token::lexer(s))
    }
}

#[cfg(test)]
mod tests {
    use super::{Logos, Span, Token};
    use std::assert_matches::assert_matches;

    #[test]
    fn block_string_test() {
        assert_eq!(
            Some(Token::BlockStringValue(
                "This is my multiline string!\n\nIsn't it cool? ????".to_string()
            )),
            Token::lexer(
                r#"
                    """
                        This is my multiline string!

                        Isn't it cool? ????
                    """
                "#
            )
            .next(),
        );
        assert_eq!(
            Some((Token::BlockStringValue("Testing span".to_string()), 1..19,)),
            Token::lexer(r#" """Testing span""" "#).spanned().next(),
        );
        assert_eq!(
            Some(Token::BlockStringValue(
                "Testing escaped block quote \"\"\"".to_string()
            )),
            Token::lexer(r#" """Testing escaped block quote \"""""" "#).next(),
        );
        assert_eq!(
            Some(Token::BlockStringValue(
                "Testing \n various \n newlines".to_string()
            )),
            Token::lexer("\"\"\"\nTesting \r various \r\n newlines\"\"\"").next(),
        );
        assert_eq!(
            Some(Token::Error),
            Token::lexer(r#" """This is a block string that doesn't end "#).next(),
        );
        assert_eq!(
            vec![
                Token::BlockStringValue("".to_string()),
                Token::StringValue(Ok("".to_string()))
            ],
            Token::lexer(r#" """""""" "#).collect::<Vec<Token>>(),
        );
    }

    #[test]
    fn string_test() {
        assert_eq!(
            Some(Token::StringValue(Ok(
                "This is a string with escaped characters and unicode: ????\u{ABCD}\u{10FFFF}!\n"
                    .to_string()
            ))),
            Token::lexer("\"This is a string with escaped characters and unicode: ????\\uABCD\\u{10FFFF}!\\n\"").next(),
        );
        assert_eq!(
            Some(Token::Error),
            Token::lexer("\"This is a string with a newline \n Not allowed!\"").next(),
        );
        assert_eq!(
            Some((Token::StringValue(Ok("Testing span".to_string())), 1..15,)),
            Token::lexer(r#" "Testing span" "#).spanned().next(),
        );
        assert_eq!(
            Some(Token::StringValue(Err(vec![Span::from(2..8)]))),
            Token::lexer(r#" "\uD800" "#).next(),
        );
        assert_eq!(
            Some(Token::StringValue(Err(vec![Span::from(2..12)]))),
            Token::lexer(r#" "\u{00D800}" "#).next(),
        );
        assert_eq!(
            Some(Token::StringValue(Ok("????".to_string()))),
            Token::lexer(r#" "\uD83D\uDD25" "#).next(),
        );
        assert_eq!(
            Some(Token::StringValue(Ok("\u{1234}\u{ABCD}".to_string()))),
            Token::lexer(r#" "\u1234\uABCD" "#).next(),
        );
        assert_eq!(
            Some(Token::StringValue(Err(vec![Span::from(2..8)]))),
            Token::lexer(r#" "\uDEAD\uDEAD" "#).next(),
        );
        assert_eq!(
            Some(Token::StringValue(Err(vec![Span::from(8..14)]))),
            Token::lexer(r#" "\uD800\uD800" "#).next(),
        );
        assert_eq!(
            Some(Token::Error),
            Token::lexer(r#" "This is a string that doesn't end "#).next(),
        );
    }

    #[test]
    fn int_test() {
        assert_eq!(
            Some(Token::IntValue(Ok(12345))),
            Token::lexer("12345").next()
        );
        assert_eq!(Some(Token::Error), Token::lexer("012345").next(),);
        assert_eq!(
            Some((Token::Error, 0..6)),
            Token::lexer("12345A").spanned().next()
        );
        assert_eq!(
            Some((Token::Error, 0..6)),
            Token::lexer("12345_").spanned().next()
        );
        assert_eq!(Some(Token::IntValue(Ok(0))), Token::lexer("0").next());
        assert_eq!(Some(Token::IntValue(Ok(0))), Token::lexer("-0").next());
        assert_matches!(
            Token::lexer((i64::from(i32::MAX) + 1).to_string().as_str()).next(),
            Some(Token::IntValue(Err(_)))
        );
        assert_matches!(
            Token::lexer((i64::from(i32::MIN) - 1).to_string().as_str()).next(),
            Some(Token::IntValue(Err(_)))
        );
    }

    #[test]
    fn float_test() {
        assert_eq!(
            Some(Token::FloatValue(Ok(12345.6789e123))),
            Token::lexer("12345.6789e123").next()
        );
        assert_eq!(
            Some(Token::FloatValue(Ok(12345e123))),
            Token::lexer("12345e123").next()
        );
        assert_eq!(
            Some(Token::FloatValue(Ok(12345.6789))),
            Token::lexer("12345.6789").next()
        );
        assert_eq!(
            Some(Token::FloatValue(Ok(0.0))),
            Token::lexer("0.00000000").next()
        );
        assert_eq!(
            Some(Token::FloatValue(Ok(-1.23))),
            Token::lexer("-1.23").next()
        );
        assert_eq!(Some(Token::Error), Token::lexer("012345.6789e123").next());
        assert_eq!(Some(Token::Error), Token::lexer("-012345.6789e123").next());
        assert_eq!(Some(Token::Error), Token::lexer("1.").next());
        assert_eq!(
            Some((Token::Error, 0..15)),
            Token::lexer("12345.6789e123A").spanned().next()
        );
    }

    #[test]
    fn name_test() {
        assert_eq!(Some(Token::Name("name")), Token::lexer("name").next());
        assert_eq!(Some(Token::Name("__name")), Token::lexer("__name").next());
        assert_eq!(Some(Token::Name("name1")), Token::lexer("name1").next());
        assert_eq!(Some(Token::Error), Token::lexer("1name").next());
        assert_eq!(
            vec![Token::Name("dashed"), Token::Error, Token::Name("name")],
            Token::lexer("dashed-name").collect::<Vec<Token>>(),
        );
    }

    #[test]
    fn comment_test() {
        assert_eq!(None, Token::lexer("# this is a comment").next());
        assert_eq!(
            Some(Token::Ampersand),
            Token::lexer("# this is a comment\n# this is another comment\r&").next(),
        );
    }
}
