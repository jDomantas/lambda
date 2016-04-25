use AstNode;

fn copy_node(node: &AstNode) -> AstNode {
	match node {
		&AstNode::Application(ref a, ref b) =>
			AstNode::Application(
				Box::new(copy_node(&**a)), 
				Box::new(copy_node(&**b))),
		&AstNode::FreeVariable(ch) =>
			AstNode::FreeVariable(ch),
		&AstNode::BoundVariable(num) =>
			AstNode::BoundVariable(num),
		&AstNode::Function(ref body) =>
			AstNode::Function(Box::new(copy_node(&**body))),
	}
}

fn increment_free(node: &AstNode, by: u32, free_threshold: u32) -> AstNode {
	match node {
		&AstNode::Application(ref a, ref b) =>
			AstNode::Application(
				Box::new(increment_free(&**a, by, free_threshold)),
				Box::new(increment_free(&**b, by, free_threshold))),
		&AstNode::BoundVariable(num) if num >= free_threshold =>
			// free variable
			AstNode::BoundVariable(num + by),
		&AstNode::BoundVariable(num) =>
			// bound variable, don't change
			AstNode::BoundVariable(num),
		&AstNode::FreeVariable(ch) =>
			AstNode::FreeVariable(ch),
		&AstNode::Function(ref body) =>
			AstNode::Function(
				Box::new(increment_free(&**body, by, free_threshold + 1))),
	}
}

fn decrement_free(node: &AstNode, free_threshold: u32) -> AstNode {
	match node {
		&AstNode::Application(ref a, ref b) =>
			AstNode::Application(
				Box::new(decrement_free(&**a, free_threshold)),
				Box::new(decrement_free(&**b, free_threshold))),
		&AstNode::BoundVariable(num) if num >= free_threshold =>
			// free variable
			AstNode::BoundVariable(num - 1),
		&AstNode::BoundVariable(num) =>
			// bound variable, don't change
			AstNode::BoundVariable(num),
		&AstNode::FreeVariable(ch) =>
			AstNode::FreeVariable(ch),
		&AstNode::Function(ref body) =>
			AstNode::Function(
				Box::new(decrement_free(&**body, free_threshold + 1))),
	}
}

fn substitute_walk(node: &AstNode, depth: u32, arg: &AstNode) -> AstNode {
	match node {
		&AstNode::FreeVariable(ch) => 
			AstNode::FreeVariable(ch),
		&AstNode::BoundVariable(num) if num == depth => {
			// this variable is bound by the parameter
			// of function that's body we are working on,
			// increment free variables in arg and return
			increment_free(arg, depth, 0)
		},
		&AstNode::BoundVariable(num) =>
			AstNode::BoundVariable(num),
		&AstNode::Function(ref body) =>
			AstNode::Function(
				Box::new(substitute_walk(&**body, depth + 1, arg))),
		&AstNode::Application(ref a, ref b) =>
			AstNode::Application(
				Box::new(substitute_walk(&**a, depth, arg)),
				Box::new(substitute_walk(&**b, depth, arg))),
	}
}

fn substitute(node: &AstNode, arg: &AstNode) -> AstNode {
	substitute_walk(node, 0, arg)
}

fn reduce_application(left: &AstNode, right: &AstNode, to_fn: bool) -> AstNode {
	let left_fn = reduce_node(left, true);
	match left_fn {
		AstNode::Function(body) => {
			// replace variables that have index of 0
			// (they are bound by this function)
			let res = substitute(&decrement_free(&body, 1), right);
			reduce_node(&res, to_fn)
		},
		_ => 
			AstNode::Application(
				Box::new(reduce_node(&left_fn, false)),
				Box::new(reduce_node(right, false)))
	}
}

fn reduce_node(node: &AstNode, to_fn: bool) -> AstNode {
	match node {
		&AstNode::Function(ref body) => {
			if to_fn {
				// already a function, just return copy
				copy_node(node)
			} else {
				// reduce recursively
				AstNode::Function(Box::new(reduce_node(&**body, false)))
			}
		}
		&AstNode::Application(ref a, ref b) =>
			reduce_application(&**a, &**b, to_fn), 
		_ => 
			// variables can't be reduced, just copy them
			copy_node(node),
	}
}

pub fn beta_reduce(node: &AstNode) -> AstNode {
	reduce_node(node, false)
}
