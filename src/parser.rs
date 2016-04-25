use std;
use std::collections::HashMap;
use AstNode;

pub struct ParseError {
	pub position: usize,
	pub message: String,
}

struct Parser<'a> {
	data: std::iter::Peekable<std::str::Chars<'a>>,
	position: usize,
	bind_depths: HashMap<char, u32>,
	current_depth: u32,
}

impl<'a> Parser<'a> {
	fn new(source: &str) -> Parser {
		Parser {
			data: source.chars().peekable(),
			position: 0,
			bind_depths: HashMap::new(),
			current_depth: 0,
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

fn map_optional_insert(map: &mut HashMap<char, u32>, key: char, value: Option<u32>) {
	match value {
		Some(val) => map.insert(key, val),
		None => map.remove(&key),
	};
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
			Ok(ch) => {
				match parser.bind_depths.get(&ch) {
					Some(depth) => Ok(AstNode::BoundVariable(
						parser.current_depth - depth)),
					None => Ok(AstNode::FreeVariable(ch)),
				}
			},
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
			
			parser.current_depth += 1;
			let old = parser.bind_depths.insert(ch, parser.current_depth);
			
			if parser.check('\\') {
				match parse_function(parser) {
					Ok(node) => {
						parser.current_depth -= 1;
						map_optional_insert(&mut parser.bind_depths, ch, old);
						Ok(AstNode::Function(Box::new(node)))
					},
					Err(e) => Err(e), 
				}
			} else {
				match parse_node(parser) {
					Ok(node) => {
						parser.current_depth -= 1;
						map_optional_insert(&mut parser.bind_depths, ch, old);
						Ok(AstNode::Function(Box::new(node)))
					},
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
