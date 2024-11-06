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
    GoFrom(String, String),       // `G` for goto label
    GoTo(String),         // `G`
    Join(String, String),         // `J` for join event
    Label(String),                // `->` for branch label
    Branch(String, String),       // Branch label and text
    GatewayJoin(String),          // `<-` for join gateway
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
    pub fn next_token(&mut self) -> Option<Token> {
        self.skip_whitespace(); // Skip any unnecessary whitespace

        match self.current_char {
            Some('=') => {
                self.advance(); // Skip '='
                if self.current_char == Some('=') {
                    self.advance(); // Skip second '=' for lanes
                    let lane_name = self.read_text();
                    Some(Token::Lane(lane_name))
                } else {
                    let pool_name = self.read_text();
                    Some(Token::Pool(pool_name))
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
                Some(event_type)
            },
            Some('-') => {
                self.advance(); // Skip '-'
                if self.current_char == Some('>') {
                    self.advance(); // Skip '>'
                    let label: String = self.read_text();
                    let text = if self.current_char == Some('"') {
                        self.advance(); // Skip '"'
                        let t: String = self.read_text();
                        self.advance(); // Skip '"'
                        t
                    } else {
                        String::new()
                    };
                    Some(Token::Branch(label, text))
                } else {
                    let text: String = self.read_text(); // Read the text after the event symbol
                    Some(Token::ActivityTask(text))
                }
            },
            Some('.') => {
                self.advance(); // Skip '.'
                let text: String = self.read_text(); // Read the text after the event symbol
                self.seen_start = false; // Reset the state for the next sequence of events
                Some(Token::EventEnd(text))
            },
            Some('<') => {
                self.advance(); // Skip '<'
                if self.current_char == Some('-') {
                    self.advance(); // Skip '-'
                    let label: String = self.read_text();
                    Some(Token::GatewayJoin(label))
                } else {
                    None
                }
            },
            Some('X') => {
                self.advance(); // Skip 'X'
                Some(Token::GatewayExclusive)
            },
            Some('G') => {
                self.advance(); // Skip 'G'
                self.skip_whitespace();
                if self.current_char == Some('-') {
                    self.advance(); // Skip '-'
                    if self.current_char == Some('>') {
                        self.advance(); // Skip '>'
                        let label: String = self.read_text();
                        let text = if self.current_char == Some('"') {
                            self.advance(); // Skip '"'
                            let t: String = self.read_text();
                            self.advance(); // Skip '"'
                            t
                        } else {
                            String::new()
                        };
                        Some(Token::GoFrom(label, text))
                    } else {
                        None
                    }
                } else if self.current_char == Some('<') {
                    self.advance(); // Skip '<'
                    if self.current_char == Some('-') {
                        self.advance(); // Skip '-'
                        let label: String = self.read_text();
                        Some(Token::GoTo(label))
                    } else {
                        None
                    }
                } else {
                    None
                }
            },
            Some('J') => {
                self.advance(); // Skip 'J'
                let label: String = self.read_text(); // Read the text after the event symbol
                let text = if self.current_char == Some('"') {
                    self.advance(); // Skip '"'
                    let t: String = self.read_text();
                    self.advance(); // Skip '"'
                    t
                } else {
                    String::new()
                };
                Some(Token::Join(label, text))
            },
            Some(c) if !c.is_whitespace() => {
                let mut text = self.read_text();
                if text.ends_with(":") {
                    text.pop(); // Remove the last character
                    Some(Token::Label(text))
                } else {
                    Some(Token::Text(text))
                }
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
            if c != '\n' && c != '-' && c != '.' && c != '"' {
                text.push(c);
                self.advance();
            } else {
                break;
            }
        }
        text.trim().to_string() // Trim any leading/trailing spaces
    }
}