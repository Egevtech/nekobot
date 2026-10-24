use bevy_reflect::prelude::*;

#[derive(Reflect, PartialEq, Debug, Clone)]
pub struct RequestConfig {
    model: String,
    thinking: String,
}

#[derive(Debug, PartialEq, Clone)]
enum Token {
    StringLiteral(String),
    Colon,
    Comma,
    Ident(String),
}

#[derive(Debug)]
enum ParseError {
    /* TOKENIZER ERRORS */
    UnexpectedIdent(String),
    UnterminatedStringLiteral,
    NewLineInStringLiteral,

    /* KVSS ERRORS */
    UnexpectedToken(Token),
    UnexpectedEndOfTokenStream,

    /* PARSE ERRORS */
    FieldIsNotDescribed(String),
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Parse error: ")?;

        match self {
            ParseError::UnexpectedIdent(s) =>
                write!(f, "Unexpected identificator: {s}"),
            ParseError::UnterminatedStringLiteral =>
                write!(f, "Unterminated string literal"),
            ParseError::NewLineInStringLiteral => 
                write!(f, "Newline in string literal"),

            ParseError::UnexpectedToken(t) => 
                write!(f, "Unexpected token: {:?}", t),
            ParseError::UnexpectedEndOfTokenStream => 
                write!(f, "Unexpected end of token stream"),

            ParseError::FieldIsNotDescribed(s) => 
                write!(f, "Field {s} is not described"),
        }
    }
}

pub type ParseResult<T> = Result<T, ParseError>;

#[derive(Debug, PartialEq)]
struct KeyValSeq {
    field: String,
    val: String,
}

impl RequestConfig {
    fn parse<T>(from: T) -> ParseResult<Self>
    where T: ToString
    {
        let tokens = Self::tokenize(from.to_string())?;
        let mut result = RequestConfig{model: String::new(), thinking: "false".to_string()};
        let kvss: Vec<KeyValSeq> = Self::make_sequenses(tokens)?;

        for i in 0..result.clone().field_len() {
            let result_clone = result.clone();
            let field_name = result_clone.name_at(i).unwrap();

            if !kvss.iter().any(|x| x.field == field_name) {
                return Err(ParseError::FieldIsNotDescribed(field_name.to_string()));
            }

            if let Some(field_value) = result.field_at_mut(i) {
                if let Some(val) = field_value.try_downcast_mut::<String>() {
                    let a = kvss.iter().find(|x| x.field == field_name).unwrap();
                    *val = a.val.clone();
                }
            }
        }

        Ok(result)
    }

    fn make_sequenses(tokens: Vec<Token>) -> ParseResult<Vec<KeyValSeq>> {
        let mut kvss: Vec<KeyValSeq> = vec![];
        let mut i = 0usize;

        while i < tokens.len() {
            if let Token::Ident(s) = &tokens[i] {
                let field_name: String;
                let field_value: String;

                field_name = s.clone();

                i += 1;
                if i > tokens.len() {
                    return Err(ParseError::UnexpectedToken(tokens[i - 1].clone()));
                }

                if tokens[i] != Token::Colon {
                    return Err(ParseError::UnexpectedToken(tokens[i].clone()));
                }

                i += 1;
                if i > tokens.len() {
                    return Err(ParseError::UnexpectedEndOfTokenStream);
                }
                if let Token::StringLiteral(a) = tokens[i].clone() {
                    field_value = a;
                } else {
                    return Err(ParseError::UnexpectedToken(tokens[i].clone()));
                }

                kvss.push(
                    KeyValSeq { field: field_name, val: field_value }
                );

                i += 1;
            } else {
                return Err(ParseError::UnexpectedToken(tokens[i].clone()));
            }
        }

        Ok(kvss)
    }

    fn tokenize(str: String) -> ParseResult<Vec<Token>> {
        let from = str.to_string();
        let chars = from.chars().collect::<Vec<char>>();

        let mut token_stream: Vec<Token> = vec![];
        let mut i: usize = 0;
        while i < chars.len() {
            if chars[i].is_alphabetic() {
                let mut ident_pull: String = String::new();
                while chars[i].is_alphanumeric() && i < chars.len() {
                    ident_pull.push(chars[i]);
                    i += 1;
                }
                token_stream.push(Token::Ident(ident_pull))
            } else if chars[i] == ':' {
                token_stream.push(Token::Colon);
            } else if chars[i] == ',' {
                token_stream.push(Token::Comma);
            } else if chars[i].is_alphanumeric() {
                return Err(ParseError::UnexpectedIdent("Indentificator can't start with number".to_string()))
            } else if chars[i] == '"' {
                i += 1;
                let start = i;

                while i < chars.len() && chars[i] != '"' {
                    if chars[i] == '\n' {
                        return Err(ParseError::NewLineInStringLiteral);
                    }
                    i += 1;
                }

                if i >= chars.len() {
                    return Err(ParseError::UnterminatedStringLiteral);
                }

                token_stream.push(Token::StringLiteral(chars[start..i].iter().collect()));

                i += 1
            } else if chars[i].is_whitespace() {
            } else {
                return Err(ParseError::UnexpectedIdent("Что это за хуйня блять вообще".to_string()));
            }

            i += 1;
        }

        Ok(token_stream)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_config_parse_test() {
        let stri = r#"model : "qwen-2.5", thinking : "false""#;
        let stru = RequestConfig{model: "qwen-2.5".to_string(), thinking: "false".to_string()};

        assert_eq!(RequestConfig::parse(stri).unwrap(), stru);
    }

    #[test]
    fn request_config_tokenizer_test() {
        let stri = r#"model : "qwen-2.5" , "#.to_string();
        let tokens = vec![Token::Ident("model".to_string()), Token::Colon, Token::StringLiteral("qwen-2.5".to_string()), Token::Comma];

        assert_eq!(RequestConfig::tokenize(stri).unwrap(), tokens)
    }

    #[test]
    fn request_config_tokenizer_unterminated_string_literal_test() {
        let stri = String::from(r#"model : "qw"#);

        assert!(RequestConfig::tokenize(stri).is_err())
    }

    #[test]
    fn request_config_tokenizer_newline_in_string_literal_test() {
        let stri = String::from("\"popa\nas\"");

        assert!(RequestConfig::tokenize(stri).is_err())
    }

    #[test]
    fn request_config_kvss_test() {
        let str = String::from(r#"popa : "zhopa""#);

        assert_eq!(
            RequestConfig::make_sequenses(RequestConfig::tokenize(str).unwrap()).unwrap(),
            vec![KeyValSeq{field: "popa".to_string(), val: "zhopa".to_string()}]
        )
    }
}