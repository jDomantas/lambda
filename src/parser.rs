use std;

pub enum AstNode {
	Variable(char),
	Application(Box<AstNode>, Box<AstNode>),
	Function(char, Box<AstNode>),
}

struct Parser<'a> {
	data: std::iter::Peekable<std::str::Chars<'a>>,
	position: usize,
	error: Option<(usize, String)>,
}

impl<'a> Parser<'a> {
	fn new(source: &str) -> Parser {
		Parser {
			data: source.chars().peekable(),
			position: 0,
			error: None,
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
		println!("advanced, now at {}", self.position);
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
	
	fn consume(&mut self, expected: char) -> bool {
		match self.check(expected) {
			true => true,
			false => {
				self.error = Some((self.position, format!("expected '{}'", expected)));
				false
			},
		}
	}
	
	fn consume_letter(&mut self) -> Option<char> {
		match self.peek() {
			Some(c) if c >= 'a' && c <= 'z' => {
				self.advance();
				Some(c)
			},
			_ => {
				self.error = Some((self.position, "expected letter".to_string()));
				None
			},
		}
	}
}

fn parse_unit(parser: &mut Parser) -> Option<AstNode> {
	if parser.check('(') {
		if let Some(node) = parse_node(parser) {
			if parser.consume(')') {
				Some(node)
			} else {
				None
			}
		} else {
			None
		}
	} else if let Some(var) = parser.consume_letter() {
		Some(AstNode::Variable(var))
	} else {
		None
	}
}

fn parse_function(parser: &mut Parser) -> Option<AstNode> {
	if let Some(parameter) = parser.consume_letter() {
		if !parser.consume('.') {
			return None;
		}
		
		if parser.check('\\') {
			match parse_function(parser) {
				Some(node) => Some(AstNode::Function(parameter, Box::new(node))),
				None => None, 
			}
		} else {
			match parse_node(parser) {
				Some(node) => Some(AstNode::Function(parameter, Box::new(node))),
				None => None,
			}
		}
	} else {
		None
	}
}

fn parse_node(parser: &mut Parser) -> Option<AstNode> {
	if parser.check('\\') {
		return parse_function(parser);
	}
	
	if let Some(unit) = parse_unit(parser) {
		let mut result = unit;
		loop {
			match parser.peek() {
				Some(c) if c == '(' || (c >= 'a' && c <= 'z') => {
					if let Some(unit) = parse_unit(parser) {
						result = AstNode::Application(Box::new(result), Box::new(unit));
					} else {
						return None;
					}
				},
				_ => break, 
			}
		}
		
		Some(result)
	} else {
		None
	}
}

pub fn parse_object<'a>(source: &'a str) -> Result<AstNode, (usize, String)> {
	let mut parser = Parser::new(source);
	
	// skip initial whitespace
	match parser.peek() {
		Some(' ') | Some('\t') | Some('\n') | Some('\r') => { 
			parser.advance();
		},
		_ => { },
	}
	
	match parse_node(&mut parser) {
		Some(node) => {
			match parser.peek() {
				None => Ok(node),
				_ => Err((parser.position, "expected end of input".to_string())),
			}
		},
		None => Err(parser.error.unwrap()),
	}
}
