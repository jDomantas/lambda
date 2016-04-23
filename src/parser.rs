use std;

pub enum AstNode {
	Variable(char),
	Application(Box<AstNode>, Box<AstNode>),
	Function(char, Box<AstNode>),
}

pub struct ParseError {
	pub position: usize,
	pub message: String,
}

struct Parser<'a> {
	data: std::iter::Peekable<std::str::Chars<'a>>,
	position: usize,
}

impl<'a> Parser<'a> {
	fn new(source: &str) -> Parser {
		Parser {
			data: source.chars().peekable(),
			position: 0,
		}
	} 
	
	fn peek(&mut self) -> Option<char> {
		// convert &char to char
		match self.data.peek() {
			Some(c) => Some(*c),
			None => None,
		}
	}
	
	fn advance(&mut self) {
		self.position += 1;
		self.data.next();
		// skip whitespace
		loop {
			match self.peek() {
				Some(' ') | Some('\t') | Some('\n') | Some('\r') => { 
					self.position += 1; 
					self.data.next();
				},
				_ => break,
			}
		}
	}
	
	fn check(&mut self, expected: char) -> bool {
		match self.peek() {
			Some(c) if c == expected => { self.advance(); true },
			_ => false,
		}
	}
	
	fn consume(&mut self, expected: char) -> Option<ParseError> {
		match self.check(expected) {
			true => None,
			false => Some(ParseError { 
				position: self.position, 
				message: format!("expected '{}'", expected) 
			}),
		}
	}
	
	fn consume_letter(&mut self) -> Result<char, ParseError> {
		match self.peek() {
			Some(c) if c >= 'a' && c <= 'z' => {
				self.advance();
				Ok(c)
			},
			_ => Err(ParseError { 
				position: self.position, 
				message: "expected letter".to_string(), 
			}),
		}
	}
}

fn parse_unit(parser: &mut Parser) -> Result<AstNode, ParseError> {
	if parser.check('(') {
		match parse_node(parser) {
			Ok(node) => match parser.consume(')') {
				None => Ok(node),
				Some(e) => Err(e),
			},
			Err(e) => Err(e),
		}
	} else {
		match parser.consume_letter() {
			Ok(ch) => Ok(AstNode::Variable(ch)),
			Err(e) => Err(e),
		}
	}
}

fn parse_function(parser: &mut Parser) -> Result<AstNode, ParseError> {
	match parser.consume_letter() {
		Ok(ch) => {
			if let Some(e) = parser.consume('.') {
				return Err(e);
			}
			
			if parser.check('\\') {
				match parse_function(parser) {
					Ok(node) => Ok(AstNode::Function(ch, Box::new(node))),
					Err(e) => Err(e), 
				}
			} else {
				match parse_node(parser) {
					Ok(node) => Ok(AstNode::Function(ch, Box::new(node))),
					Err(e) => Err(e), 
				}
			}
		},
		Err(e) => Err(e),
	}
}

fn parse_node(parser: &mut Parser) -> Result<AstNode, ParseError> {
	if parser.check('\\') {
		return parse_function(parser);
	}
	
	match parse_unit(parser) {
		Ok(unit) => {
			let mut result = unit;
			loop {
				match parser.peek() {
					Some(c) if c == '(' || (c >= 'a' && c <= 'z') => {
						match parse_unit(parser) {
							Ok(unit) => result = AstNode::Application(
								Box::new(result), 
								Box::new(unit)),
							Err(e) => return Err(e),
						}
					},
					_ => break, 
				}
			}
			
			Ok(result)
		},
		Err(e) => Err(e),
	}
}

pub fn parse_object(source: &str) -> Result<AstNode, ParseError> {
	let mut parser = Parser::new(source);
	
	// skip initial whitespace
	match parser.peek() {
		Some(' ') | Some('\t') | Some('\n') | Some('\r') => { 
			parser.advance();
		},
		_ => { },
	}
	
	match parse_node(&mut parser) {
		Ok(node) => match parser.peek() {
			None => Ok(node),
			Some(_) => Err(ParseError {
				position: parser.position,
				message: "expected end of input".to_string(),
			}),
		},
		Err(e) => Err(e),
	}
}
