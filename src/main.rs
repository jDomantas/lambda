mod parser;

use std::io;
use parser::AstNode;

fn print_node(node: AstNode) {
	match node {
		AstNode::Variable(v) => print!("{}", v),
		AstNode::Function(p, body) => {
			print!("(\\{}.", p);
			print_node(*body);
			print!(")");
		},
		AstNode::Application(a, b) => {
			print!("(");
			print_node(*a);
			print_node(*b);
			print!(")");
		}
	}
}

fn report_error(input: &str, position: usize, msg: String) {
	print!("{}", input);
	for _ in 0..position {
		print!(" ");
	}
	println!("^");
	println!("Error (column {}): {}", position, msg);
}

fn main() {
	let mut input = String::new();
	
	io::stdin().read_line(&mut input).expect("Failed to read line");
	
	match parser::parse_object(&input) {
		Ok(node) => {
			println!("Success!");
			print_node(node);
			println!("");
		}
		Err((pos, e)) => report_error(&input, pos, e),
	}
}
