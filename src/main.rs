mod parser;
mod reduction;

use std::io;
use parser::ParseError;

pub enum AstNode {
	FreeVariable(char),
	BoundVariable(u32),
	Application(Box<AstNode>, Box<AstNode>),
	Function(Box<AstNode>),
}

fn pretty_print_walk(node: &AstNode, current_depth: u32, in_application: bool) {
	match node {
		&AstNode::Application(ref a, ref b) => {
			pretty_print_walk(&**a, current_depth, true);
			match **b {
				AstNode::Application(..) => {
					print!("(");
					pretty_print_walk(&**b, current_depth, true);
					print!(")");
				},
				_ => {
					pretty_print_walk(&**b, current_depth, true);
				},
			}
		},
		&AstNode::BoundVariable(num) => {
			let ch = std::char::from_u32(
				current_depth - num - 1 + ('a' as u32));
			print!("{}", ch.unwrap_or('?'));
		},
		&AstNode::FreeVariable(ch) => {
			print!("{}", ch);
		},
		&AstNode::Function(ref body) => {
			let param = std::char::from_u32(current_depth + ('a' as u32))
				.unwrap_or('?');
			if in_application {
				print!("(\\{}.", param);
			} else {
				print!("\\{}.", param);
			}
			pretty_print_walk(&**body, current_depth + 1, false);
			if in_application {
				print!(")");
			}
		},
	}
}

fn pretty_print(node: &AstNode) {
	pretty_print_walk(node, 0, false);
}

fn print_node(node: &AstNode) {
	match node {
		&AstNode::FreeVariable(ch) => print!("{}", ch),
		&AstNode::BoundVariable(v) => print!("{}", v),
		&AstNode::Function(ref body) => {
			print!("(\\");
			print_node(&**body);
			print!(")");
		},
		&AstNode::Application(ref a, ref b) => {
			print!("(");
			print_node(&**a);
			print!(" ");
			print_node(&**b);
			print!(")");
		}
	}
}

fn report_error(input: &str, err: ParseError) {
	println!("{}", input);
	for _ in 0..err.position {
		print!(" ");
	}
	println!("^");
	// + 1 because editors index columns starting from 1
	println!("Error (column {}): {}", err.position + 1, err.message);
}

fn main() {
	let mut input = String::new();
	
	io::stdin().read_line(&mut input).expect("Failed to read line");
	// when reading from stdin strings always
	// have a trailing newline for some reason
	assert_eq!(input.pop().unwrap_or('\0'), '\n');
	
	match parser::parse_object(&input) {
		Ok(node) => {
			println!("Success!");
			pretty_print(&node);
			println!("");
			let reduced = reduction::beta_reduce(&node);
			pretty_print(&reduced);
			println!("");
		}
		Err(e) => report_error(&input, e),
	}
}
