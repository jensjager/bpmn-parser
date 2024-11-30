// src/lexer.rs

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Pool(String),                 // `=` for pool
    Lane(String),                 // `==` for lane
    EventStart(String),           // `#` for start event
    EventMiddle(String),          // `#` for middle event (detected by context)
    EventEnd(String),             // `.` for end event
    ActivityTask(String),         // `-` for task activity
    GatewayExclusive,             // `X` for gateway
    Go,                           // `G` for go 
    Join(String, String),         // `J` for join event
    Label(String),                // `:` for branch label
    Branch(String, String),       // `->` Branch label and text
    JoinLabel(String),            // `<-` for join gateway
    Text(String),                 // Any freeform text
    Eof,                          // End of file/input
}

#[derive(Debug, Clone)]
pub enum LexerError {
    UnexpectedCharacter(char, usize, usize, String), // character, line, column, highlight
    UnterminatedString(usize, usize, String),        // line, column, highlight
}

impl std::fmt::Display for LexerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LexerError::UnexpectedCharacter(c, line, col, highlight) => {
                write!(f, "Unexpected character '{}' at line {}, column {}\n{}", c, line, col, highlight)
            }
            LexerError::UnterminatedString(line, col, highlight) => {
                write!(f, "Unterminated quoted string starting at line {}, column {}\n{}", line, col, highlight)
            }
        }
    }
}

#[derive(Clone)]
pub struct Lexer<'a> {
    input: &'a str,                 // Input string
    position: usize,                // Current position in the input
    current_char: Option<char>,     // Current character being examined
    pub line: usize,                // Current line number
    column: usize,                  // Current column number
    pub seen_start: bool,               // State flag for distinguishing event start/middle
}

impl<'a> Lexer<'a> {
    // Create a new lexer from an input string
    pub fn new(input: &'a str) -> Self {
        let mut lexer = Lexer {
            input,
            position: 0,
            current_char: None,
            line: 1,
            column: 0,
            seen_start: false,    // Initially, no start event has been seen
        };
        lexer.advance(); // Load the first character
        lexer
    }

    // Advance to the next character in the input
    fn advance(&mut self) {
        if self.current_char == Some('\n') {
            self.line += 1;          // Move to the next line
            self.column = 1;         // Reset the column count
        } else {
            self.column += 1;        // Move to the next column
        }

        if self.position < self.input.len() {
            self.current_char = Some(self.input.chars().nth(self.position).unwrap());
            self.position += 1;
        } else {
            self.current_char = None; // End of input
        }
    }

    // Peek the next token in the input
    pub fn peek_token(&mut self) -> Result<Token, LexerError> {
        let saved_position = self.position;
        let saved_char = self.current_char;
        let saved_line = self.line;
        let saved_column = self.column;
    
        // Get the next token
        let token = self.next_token();
    
        // Restore lexer state
        self.position = saved_position;
        self.current_char = saved_char;
        self.line = saved_line;
        self.column = saved_column;
    
        token
    }

    // Get the next token from the input
    pub fn next_token(&mut self) -> Result<Token, LexerError> {
        self.skip_whitespace(); // Skip any unnecessary whitespace

        match self.current_char {
            Some('/') => {
                self.advance(); // Skip '/'
                if self.current_char == Some('/') {
                    while self.current_char != Some('\n') {
                        self.advance(); // Skip the comment
                    }
                    self.advance(); // Skip the newline
                    self.next_token()
                } else {
                    Err(LexerError::UnexpectedCharacter('/', self.line, self.column, self.highlight_error()))
                }
            },
            Some('=') => {
                self.advance(); // Skip '='
                if self.current_char == Some('=') {
                    self.advance(); // Skip second '=' for lanes
                    let lane_name = self.read_text();
                    Ok(Token::Lane(lane_name))
                } else {
                    let pool_name = self.read_text();
                    Ok(Token::Pool(pool_name))
                }
            },
            Some('#') => {
                self.advance(); // Skip '#'
                let text: String = self.read_text(); // Read the text after the event symbol
                let event_type = if !self.seen_start { // First '-' is a Start event
                    self.seen_start = true;  // Mark that we've seen a start event
                    Token::EventStart(text)
                } else {
                    Token::EventMiddle(text) // Subsequent '-' are Middle events
                };
                Ok(event_type)
            },
            Some('-') => {
                self.advance(); // Skip '-'
                if self.current_char == Some('>') {
                    self.advance(); // Skip '>'
                    let label: String = self.read_text();
                    let text = self.read_quoted_text();
                    Ok(Token::Branch(label, text?))
                } else {
                    let text: String = self.read_text(); // Read the text after the event symbol
                    Ok(Token::ActivityTask(text))
                }
            },
            Some('.') => {
                self.advance(); // Skip '.'
                let text: String = self.read_text(); // Read the text after the event symbol
                self.seen_start = false; // Reset the state for the next sequence of events
                Ok(Token::EventEnd(text))
            },
            Some('<') => {
                self.advance(); // Skip '<'
                if self.current_char == Some('-') {
                    self.advance(); // Skip '-'
                    let label: String = self.read_text();
                    Ok(Token::JoinLabel(label))
                } else {
                    Err(LexerError::UnexpectedCharacter(self.current_char.unwrap_or('\0'), self.line, self.column, self.highlight_error()))
                }
            },
            Some('X') => {
                self.advance(); // Skip 'X'
                Ok(Token::GatewayExclusive)
            },
            Some('G') => {
                self.advance(); // Skip 'G'
                Ok(Token::Go)
            },
            Some('J') => {
                self.advance(); // Skip 'J'
                let label: String = self.read_text(); // Read the text after the event symbol
                let text = self.read_quoted_text();
                Ok(Token::Join(label, text?))
            },
            Some(c) if !c.is_whitespace() => {
                let mut text = self.read_text();
                if text.ends_with(":") {
                    text.pop(); // Remove the last character
                    Ok(Token::Label(text))
                } else {
                    Ok(Token::Text(text))
                }
            },
            None => Ok(Token::Eof), // End of input
            _ => {
                self.advance();
                self.next_token()
            },
        }
    }

    // Skip over any whitespace
    fn skip_whitespace(&mut self) {
        while let Some(c) = self.current_char {
            if c.is_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }

    // Read freeform text (e.g., event labels or descriptions)
    fn read_text(&mut self) -> String {
        let mut text = String::new();

        while let Some(c) = self.current_char {
            if c != '\n' && c != '-' && c != '.' && c != '#' && c != '"' {
                text.push(c);
                self.advance();
            } else {
                break;
            }
        }
        text.trim().to_string() // Trim any leading/trailing spaces
    }

    fn read_quoted_text(&mut self) -> Result<String, LexerError> {
        if self.current_char == Some('"') {
            self.advance(); // Skip the opening quote
            let text: String = self.read_text();
            if self.current_char == Some('"') {
                self.advance(); // Skip the closing quote
            } else {
                return Err(LexerError::UnterminatedString(self.line, self.column, self.highlight_error()));
            }
            Ok(text)
        } else {
            Ok(String::new())
        }
    }

    pub fn highlight_error(&self) -> String {
        // Split input into lines
        let lines: Vec<&str> = self.input.split('\n').collect();
    
        let current_line = lines[self.line - 1];
    
        // Create the error highlight
        let highlight = " ".repeat(self.column - 2) + "^";
    
        format!("{}\n{}", current_line, highlight)
    }
}