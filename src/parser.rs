use std;
use std::collections::HashMap;
use AstNode;

pub struct ParseError {
	pub position: usize,
	pub message: String,
}

enum TokenContents {
	Letter(char),
	Number(u32),
	Name(String),
	Dot,
	Lambda,
	OpenParenth,
	CloseParenth,
	End,
}

struct Token {
	position: usize,
	contents: TokenContents,
}

struct Lexer<'a> {
	data: std::iter::Peekable<std::str::Chars<'a>>,
	position: usize,
}

fn is_whitespace(ch: char) -> bool {
	match ch {
		' ' | '\t' | '\r' | '\n' => true,
		_ => false,
	}
}

fn is_name(ch: char) -> bool {
	ch >= 'A' && ch <= 'Z'
}

fn is_variable(ch: char) -> bool {
	ch >= 'a' && ch <= 'z'
}

fn is_digit(ch: char) -> bool {
	ch >= '0' && ch <= '9'
}

impl<'a> Lexer<'a> {
	fn new(source: &str) -> Lexer {
		Lexer {
			data: source.chars().peekable(),
			position: 0,
		}
	}
	
	fn peek_char(&mut self) -> Option<char> {
		// convert &char to char
		match self.data.peek() {
			Some(c) => Some(*c),
			None => None,
		}
	}
	
	fn advance(&mut self) {
		self.data.next();
		self.position += 1;
	}
	
	fn skip_whitespace(&mut self) {
		loop {
			match self.peek_char() {
				Some(ch) if is_whitespace(ch) => self.advance(),
				_ => break,
			}
		}
	}
	
	fn punctuation_token(&mut self, contents: TokenContents) -> Token {
		let start = self.position;
		self.advance();
		Token {
			position: start,
			contents: contents,
		}
	}
	
	fn name_token(&mut self, start: usize) -> Result<Token, ParseError> {
		let mut name = String::new();
		loop {
			match self.peek_char() {
				Some(ch) if is_name(ch) || is_digit(ch) => {
					name.push(ch);
					self.advance();
				},
				Some(ch) if is_variable(ch) => return Err(ParseError {
					position: start,
					message: "names must consist of capital \
					          letters and numbers".to_string(),
				}),
				_ => return Ok(Token {
					position: start,
					contents: TokenContents::Name(name),
				}),
			}
		}
	}
	
	fn number_token(&mut self, start: usize) -> Result<Token, ParseError> {
		let mut accumulator: u64 = 0;
		loop {
			match self.peek_char() {
				Some(ch) if is_digit(ch) => {
					accumulator = accumulator * 10 + 
						(ch as u64) - ('0' as u64);
					if accumulator > (std::u32::MAX as u64) {
						return Err(ParseError {
							position: start,
							message: "integer literal is too large".to_string(),
						});
					}
					self.advance();
				}
				Some(ch) if is_variable(ch) || is_name(ch) => 
					return Err(ParseError {
						position: self.position,
						message: "invalid number".to_string(),
					}),
				_ => return Ok(Token {
					position: start,
					contents: TokenContents::Number(accumulator as u32),
				}),
			}
		} 
	}
	
	fn variable_token(&mut self, start: usize) -> Result<Token, ParseError> {
		// this is called when initial symbol is already
		// found, so unwrapping should be safe
		let var = self.peek_char().unwrap();
		// skip that initial symbol
		self.advance();
		match self.peek_char() {
			Some(ch) if is_digit(ch) => Err(ParseError {
				position: self.position,
				message: "variable can't be immediately \
				          followed by a number".to_string(),
			}),
			Some(ch) if is_name(ch) => Err(ParseError {
				position: self.position,
				message: "variable can't be immediately \
				          followed by a name".to_string(),
			}),
			_ => Ok(Token {
				position: start,
				contents: TokenContents::Letter(var),
			})
		}
	}
	
	fn next_token(&mut self) -> Result<Token, ParseError> {
		self.skip_whitespace();
		let token_start = self.position;
		match self.peek_char() {
			None => Ok(Token { 
				position: token_start,
				contents: TokenContents::End, 
			}),
			Some(ch) => match ch {
				n if is_digit(n) => self.number_token(token_start),
				l if is_variable(l) => self.variable_token(token_start),
				n if is_name(n) => self.name_token(token_start),
				'.' => Ok(self.punctuation_token(TokenContents::Dot)),
				'\\' => Ok(self.punctuation_token(TokenContents::Lambda)),
				'(' => Ok(self.punctuation_token(TokenContents::OpenParenth)),
				')' => Ok(self.punctuation_token(TokenContents::CloseParenth)),
				_ => Err(ParseError {
					position: token_start,
					message: "invalid token".to_string(),
				}),
			},
		}
	}
}

struct Parser<'a> {
	lexer: Lexer<'a>,
	next_token: Token,
	has_token: bool,
	bind_depths: HashMap<char, u32>,
	current_depth: u32,
}

impl<'a> Parser<'a> {
	fn new(source: &str) -> Parser {
		Parser {
			lexer: Lexer::new(source),
			next_token: Token { position: 0, contents: TokenContents::End },
			has_token: false,
			bind_depths: HashMap::new(),
			current_depth: 0,
		}
	} 
	
	fn peek(&mut self) -> Result<&Token, ParseError> {
		if !self.has_token {
			self.next_token = try!(self.lexer.next_token());
			self.has_token = true;
		}
		
		Ok(&(self.next_token))
	}
	
	fn consume(&mut self) -> Result<Token, ParseError> {
		if !self.has_token {
			try!(self.peek());
		}
		
		// move out old token, replace with some random unused value
		let old_position = self.next_token.position;
		let token = std::mem::replace(
			&mut self.next_token,
			Token {
				position: old_position,
				contents: TokenContents::End 
			});
		
		self.has_token = false;
		return Ok(token);
	}
	
	fn error(&self, message: String) -> ParseError {
		ParseError {
			position: self.next_token.position,
			message: message,
		}
	}
}

fn create_church_numeral(num: u32) -> AstNode {
	let mut node = AstNode::BoundVariable(0);
	for _ in 0..num {
		node = AstNode::Application(
			Box::new(AstNode::BoundVariable(1)),
			Box::new(node));
	}
	
	return AstNode::Function(Box::new(AstNode::Function(Box::new(node))));
}

fn map_optional_insert(map: &mut HashMap<char, u32>, key: char, value: Option<u32>) {
	match value {
		Some(val) => map.insert(key, val),
		None => map.remove(&key),
	};
}

fn parse_unit(parser: &mut Parser) -> Result<AstNode, ParseError> {
	let token = try!(parser.consume());
	match token.contents {
		TokenContents::OpenParenth => {
			let node = try!(parse_node(parser));
			let close_parenth = try!(parser.consume());
			match close_parenth.contents {
				TokenContents::CloseParenth => Ok(node),
				_ => Err(parser.error("expected name, letter, number, (, or )".to_string())),
			}
		},
		TokenContents::Number(num) => {
			Ok(create_church_numeral(num))
		},
		TokenContents::Letter(ch) => {
			match parser.bind_depths.get(&ch) {
				Some(depth) => Ok(AstNode::BoundVariable(
					parser.current_depth - depth)),
				None => Ok(AstNode::FreeVariable(ch)),
			}
		},
		TokenContents::Name(s) => {
			Ok(AstNode::Name(s))
		},
		_ => {
			Err(parser.error("expected name, letter, number, or (".to_string()))
		},
	}
}

fn parse_function(parser: &mut Parser) -> Result<AstNode, ParseError> {
	let token = try!(parser.consume());
	match token.contents {
		TokenContents::Letter(ch) => {
			parser.current_depth += 1;
			let old = parser.bind_depths.insert(ch, parser.current_depth);
			
			let body;
			match try!(parser.peek()).contents {
				TokenContents::Dot => {
					// we have just checked that this is 
					// a dot token, so it can't be error
					assert!(parser.consume().is_ok());
					body = try!(parse_node(parser));
				},
				TokenContents::Letter(..) => {
					body = try!(parse_function(parser));
				},
				_ => {
					return Err(parser.error("expected letter or .".to_string()));
				},
			}
			
			parser.current_depth -= 1;
			map_optional_insert(&mut parser.bind_depths, ch, old);
			Ok(AstNode::Function(Box::new(body)))
		},
		_ => Err(parser.error("expected letter".to_string())),
	}
}

fn parse_node(parser: &mut Parser) -> Result<AstNode, ParseError> {
	match try!(parser.peek()).contents {
		TokenContents::Lambda => {
			// we have just checked that this is 
			// a dot token, so it can't be error
			assert!(parser.consume().is_ok());
			return parse_function(parser);
		}
		_ => (),
	}
	
	let mut result = try!(parse_unit(parser));
	
	loop {
		match try!(parser.peek()).contents {
			TokenContents::OpenParenth |
			TokenContents::Letter(..) |
			TokenContents::Number(..) |
			TokenContents::Name(..) => {
				let next_unit = try!(parse_unit(parser));
				result = AstNode::Application(
					Box::new(result),
					Box::new(next_unit));
			},
			_ => break, 
		}
	}
			
	Ok(result)
}

pub fn parse_object(source: &str) -> Result<AstNode, ParseError> {
	let mut parser = Parser::new(source);	
	let node = try!(parse_node(&mut parser));
	
	match try!(parser.peek()).contents {
		TokenContents::End => Ok(node),
		_ => Err(parser.error("expected end of input".to_string())),
	}
}
