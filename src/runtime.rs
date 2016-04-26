use std::collections::HashMap;
use AstNode;
use pretty_print;
use parser;
use reduction;

pub struct Interpreter {
	named_fns: HashMap<String, AstNode>
}

fn is_name(ch: char) -> bool {
	ch >= 'A' && ch <= 'Z' 
}

fn is_digit(ch: char) -> bool {
	ch >= '0' && ch <= '9'
}

fn split_by_binding(line: &str) -> Option<(String, String)> {
	let mut last: char = ' ';
	let mut index: usize = 0;
	for ch in line.chars() {
		if ch == '=' && last == ':' {
			// split to two strings
			
			// iterators are painful to pass to other 
			// functions for some reason
			let mut first = String::new();
			for ch in line.chars().take(index - 1) {
				first.push(ch);
			}
			let mut second = String::new();
			for ch in line.chars().skip(index + 1) {
				second.push(ch);
			}
			return Some((first, second));
		}
		
		index += 1;
		last = ch;
	}
	
	None
}

fn print_parse_error(input: &str, err: parser::ParseError) {
	println!("{}", input);
	for _ in 0..err.position {
		print!(" ");
	}
	println!("^");
	// + 1 because editors index columns starting from 1
	println!("Error (column {}): {}", err.position + 1, err.message);
}

fn is_valid_name(name: &str) -> bool {
	if name.is_empty() {
		println!("name cannot be empty");
		return false;
	}
	
	for ch in name.chars() {
		if !is_digit(ch) && !is_name(ch) {
			println!("invalid name");
			return false;
		} 
	}
	
	// we checked earlier that the string is not empty
	if !is_name(name.chars().nth(0).unwrap()) {
		println!("invalid name");
		return false;
	}
	
	true
}

fn numeric_value(node: &AstNode) -> Option<u32> {
	let mut result = 0u32;
	let mut current_node = node;
	
	for _ in 0..2 {
		match current_node {
			&AstNode::Function(ref body) => current_node = &**body,
			_ => return None,
		}
	}
	
	loop {
		match current_node {
			&AstNode::BoundVariable(0) => return Some(result),
			&AstNode::Application(ref f, ref x) => {
				match &**f {
					&AstNode::BoundVariable(1) => (),
					_ => return None,
				}
				current_node = x;
				result += 1;
			},
			_ => return None,
		}
	}
}

impl Interpreter {
	pub fn new() -> Interpreter {
		Interpreter {
			named_fns: HashMap::new(),
		}
	}
	
	fn replace_named_functions(&self, obj: &AstNode) -> Result<AstNode, ()> {
		match obj {
			&AstNode::Application(ref a, ref b) =>
				Ok(AstNode::Application(
					Box::new(try!(self.replace_named_functions(&**a))),
					Box::new(try!(self.replace_named_functions(&**b))))),
			&AstNode::BoundVariable(v) =>
				Ok(AstNode::BoundVariable(v)),
			&AstNode::FreeVariable(v) =>
				Ok(AstNode::FreeVariable(v)),
			&AstNode::Function(ref body) =>
				Ok(AstNode::Function(
					Box::new(try!(self.replace_named_functions(&**body))))),
			&AstNode::Name(ref name) => {
				match self.named_fns.get(&**name) {
					None => {
						println!("[Error] unknown function: {}", name);
						Err( () )
					},
					Some(ref node) => {
						Ok(try!(self.replace_named_functions(node)))
					},
				}
			}
		}
	}

	fn process_object(&self, obj: AstNode) -> bool {
		let replaced = self.replace_named_functions(&obj);
		match replaced {
			Err(..) => false,
			Ok(node) => {
				let reduced = reduction::beta_reduce(&node);
				println!("beta-reduced to:");
				pretty_print(&reduced);
				println!("");
				match numeric_value(&reduced) {
					Some(num) => println!("Church numeral for: {}", num),
					None => println!("Not a Church numeral"),
				}
				true
			}
		}
	}

	pub fn eval_line(&mut self, line: &str) -> bool {
		if let Some((name, expr)) = split_by_binding(line) {
			let name: String = name.trim().to_string();
			if !is_valid_name(&name) {
				return false
			}
			
			match parser::parse_object(&expr) {
				Ok(obj) => {
					print!("bound {} to ", name);
					pretty_print(&obj);
					println!("");
					self.named_fns.insert(name, obj);
					true
				},
				Err(e) => {
					print_parse_error(line, e);
					false
				}
			}
		} else {
			match parser::parse_object(line) {
				Ok(obj) => {
					self.process_object(obj)
				},
				Err(e) => {
					print_parse_error(line, e);
					false
				}
			} 
		}
	}
}
