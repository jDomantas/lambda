mod parser;

use std::io;
use parser::ParseError;

pub enum AstNode {
	FreeVariable(char),
	BoundVariable(u32),
	Application(Box<AstNode>, Box<AstNode>),
	Function(Box<AstNode>),
}

fn print_node(node: &AstNode) {
	match node {
		&AstNode::FreeVariable(ch) => print!("({})", ch),
		&AstNode::BoundVariable(v) => print!("({})", v),
		&AstNode::Function(ref body) => {
			print!("(\\");
			print_node(&**body);
			print!(")");
		},
		&AstNode::Application(ref a, ref b) => {
			print!("(");
			print_node(&**a);
			print_node(&**b);
			print!(")");
		}
	}
}

fn report_error(input: &str, err: ParseError) {
	print!("{}", input);
	for _ in 0..err.position {
		print!(" ");
	}
	println!("^");
	println!("Error (column {}): {}", err.position, err.message);
}

fn main() {
	let mut input = String::new();
	
	io::stdin().read_line(&mut input).expect("Failed to read line");
	
	match parser::parse_object(&input) {
		Ok(node) => {
			println!("Success!");
			print_node(&node);
			println!("");
		}
		Err(e) => report_error(&input, e),
	}
}
