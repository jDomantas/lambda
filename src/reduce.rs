use std::collections::HashMap;
use AstNode;

// contains either a parameter and a reference
// to existing tree (to form a function), or
// a copied ast that is not a function
enum ReductionResult<'a> {
	Function(u32, &'a AstNode),
	Node(AstNode),
}

fn copy_ast(node: &AstNode) -> AstNode {
	match node {
		&AstNode::Function(p, ref b) => 
			AstNode::Function(p, Box::new(copy_ast(b))),
		&AstNode::Variable(v) => 
			AstNode::Variable(v),
		&AstNode::Application(ref a, ref b) => 
			AstNode::Application(Box::new(copy_ast(a)), Box::new(copy_ast(b))),
	}
}

/*fn reduce_to_function<'a>(node: &AstNode, map: &mut HashMap<u32, &AstNode>) 
	-> ReductionResult<'a> {
	match node {
		&AstNode::Function(p, ref b) => 
			b,//ReductionResult::Function(p, &**b),
		/*&AstNode::Variable(v) => {
			let var = map.get(&v);
			/*match map.get(&v) {
				Some(node) => reduce_to_function(node, map),
				None => ReductionResult::Node(AstNode::Variable(v)),
			}*/
			ReductionResult::Node(AstNode::Variable(v))
		},
		&AstNode::Application(a, b) => {
			/*match reduce_to_function(&*a, map) {
				// if we can't reduce to function
				// then don't do anything else
				ReductionResult::Node(node) => ReductionResult::Node(node),
				// if lhs is function, then do application
				// and reduce what's left to a function
				ReductionResult::Function(param, body) => {
					let old_value = map.insert(param, &*b);
					let reduced = reduce_to_function(&body, map);
					if old_value.is_some() {
						map.insert(param, old_value.unwrap());
					}
					reduced
				}
			}*/
		},*/
		_ => ReductionResult::Node(AstNode::Variable(1)),
	}
}*/

fn reduce_everything<'a>(node: &'a AstNode, map: &mut HashMap<u32, &'a AstNode>) -> AstNode {
	match node {
		&AstNode::Function(p, ref body) => {
			AstNode::Function(p, Box::new(reduce_everything(&*body, map)))
		},
		&AstNode::Variable(v) => {
			let map_to = match map.get(&v) {
				None => None,
				Some(node) => Some(*node),
			};
			match map_to {
				None => AstNode::Variable(v),
				Some(node) => reduce_everything(&node, map),
			}
		},
		&AstNode::Application(ref a, ref b) => {
			// TODO: change to 'reduce_to_function' later
			let lhs = reduce_everything(a, map);
			match lhs {
				AstNode::Function(p, body) => {
					let old_value = map.insert(p, &*b);
					let reduced: AstNode = reduce_everything(&*body, map);
					if old_value.is_some() { 
						map.insert(p, old_value.unwrap());
					} else {
						map.remove(&p);
					}
					reduced
					//AstNode::Variable(0)
				},
				_ => AstNode::Application(
					Box::new(lhs), 
					Box::new(reduce_everything(b, map))),
			}
		},
	}
}

pub fn beta_reduce(node: &AstNode) -> AstNode {
	let mut map = HashMap::new();
	reduce_everything(node, &mut map)
}
