mod parser;
mod reduction;
mod runtime;

use std::io;
use std::io::prelude::*;
use runtime::Interpreter;

pub enum AstNode {
	FreeVariable(char),
	BoundVariable(u32),
	Application(Box<AstNode>, Box<AstNode>),
	Function(Box<AstNode>),
	Name(String),
}

fn pretty_print_walk(node: &AstNode, current_depth: u32, in_application: bool) {
	match node {
		&AstNode::Application(ref a, ref b) => {
			pretty_print_walk(&**a, current_depth, true);
			print!(" ");
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
		&AstNode::Name(ref name) => {
			print!("{}", name);	
		},
	}
}

fn pretty_print(node: &AstNode) {
	pretty_print_walk(node, 0, false);
}

/// Prints node contents. As the parser mangles bound 
/// variable names, this prints the internal format, 
/// so it usually used for debugging.
#[allow(dead_code)]
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
		},
		&AstNode::Name(ref name) => {
			print!("{}", name);	
		},
	}
}

fn main() {
	let mut input = String::new();
	let mut interpreter = Interpreter::new();
	
	loop {
		print!("> ");
		io::stdout().flush().expect("Failed to flush stdout");
		input.clear();
		io::stdin().read_line(&mut input).expect("Failed to read line");
		// when reading from stdin strings always
		// have a trailing newline for some reason
		assert_eq!(input.pop().unwrap_or('\0'), '\n');
		
		interpreter.eval_line(&input);
	}
}
