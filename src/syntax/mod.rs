use tree_sitter::{Parser, Query, QueryCursor, StreamingIteratorMut};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenType {
    Keyword,
    Function,
    Type,
    String,
    Number,
    Comment,
    Variable,
    Punctuation,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct SyntaxToken {
    pub start_byte: usize,
    pub end_byte: usize,
    pub token_type: TokenType,
}

pub struct Highlighter {
    parser: Parser,
    query: Query,
}

impl Highlighter {
    pub fn new() -> Self {
        let mut parser = Parser::new();

        let language = tree_sitter_rust::LANGUAGE.into();
        parser
            .set_language(&language)
            .expect("Error loading Rust grammar");

        let query_source = r#"
            (function_item name: (identifier) @function)
            (type_identifier) @type
            (primitive_type) @type
            (line_comment) @comment
            (block_comment) @comment
            (string_literal) @string
            (integer_literal) @number
            (float_literal) @number
            (boolean_literal) @keyword
            (mutable_specifier) @keyword
            (visibility_modifier) @keyword
            ["fn" "let" "if" "else" "return" "struct" "impl" "match" "enum" "use" "for" "while" "loop"] @keyword
        "#;

        let query = Query::new(&language, query_source).expect("Error compiling syntax query");

        Self { parser, query }
    }

    pub fn parse_text(&mut self, text: &str) -> Vec<SyntaxToken> {
        let tree = self.parser.parse(text, None).expect("Error parsing text");

        let mut cursor = QueryCursor::new();
        let mut matches = cursor.matches(&self.query, tree.root_node(), text.as_bytes());

        let mut tokens = Vec::new();
        let capture_names = self.query.capture_names();

        while let Some(m) = matches.next_mut() {
            for capture in m.captures {
                let capture_name = capture_names[capture.index as usize];

                let token_type = match capture_name {
                    "keyword" => TokenType::Keyword,
                    "function" => TokenType::Function,
                    "type" => TokenType::Type,
                    "string" => TokenType::String,
                    "number" => TokenType::Number,
                    "comment" => TokenType::Comment,
                    _ => TokenType::Unknown,
                };

                tokens.push(SyntaxToken {
                    start_byte: capture.node.start_byte(),
                    end_byte: capture.node.end_byte(),
                    token_type,
                });
            }
        }

        tokens.sort_by_key(|t| t.start_byte);
        tokens
    }
}
