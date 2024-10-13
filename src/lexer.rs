// src/lexer.rs

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    EventStart,                   // `-` for start event
    EventMiddle,                  // `-` for middle event (detected by context)
    EventEnd,                     // `.` for end event
    Text(String),                 // Any freeform text
    Eof,                          // End of file/input
}

#[derive(Clone)]
pub struct Lexer<'a> {
    input: &'a str,                 // Input string
    position: usize,                // Current position in the input
    current_char: Option<char>,     // Current character being examined
    seen_start: bool,               // State flag for distinguishing event start/middle
}

impl<'a> Lexer<'a> {
    // Create a new lexer from an input string
    pub fn new(input: &'a str) -> Self {
        let mut lexer = Lexer {
            input,
            position: 0,
            current_char: None,
            seen_start: false,    // Initially, no start event has been seen
        };
        lexer.advance(); // Load the first character
        lexer
    }

    // Advance to the next character in the input
    fn advance(&mut self) {
        if self.position < self.input.len() {
            self.current_char = Some(self.input.chars().nth(self.position).unwrap());
            self.position += 1;
        } else {
            self.current_char = None; // End of input
        }
    }

    // Get the next token from the input
    pub fn  next_token(&mut self) -> Option<Token> {
        self.skip_whitespace(); // Skip any unnecessary whitespace

        match self.current_char {
            Some('-') => {
                self.advance(); // Skip '-'
                let event_type = if !self.seen_start { // First '-' is a Start event
                    self.seen_start = true;  // Mark that we've seen a start event
                    Token::EventStart
                } else {
                    Token::EventMiddle // Subsequent '-' are Middle events
                };
                Some(event_type)
            },
            Some('.') => {
                self.advance(); // Skip '.'
                self.seen_start = false; // Reset the state for the next sequence of events
                Some(Token::EventEnd)
            },
            Some(c) if !c.is_whitespace() => {
                let text = self.read_text();
                Some(Token::Text(text))
            },
            None => Some(Token::Eof), // End of input
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
            if c != '\n' && c != '-' && c != '.' {  // Stop at newlines or event symbols
                text.push(c);
                self.advance();
            } else {
                break;
            }
        }
        text.trim().to_string() // Trim any leading/trailing spaces
    }
}