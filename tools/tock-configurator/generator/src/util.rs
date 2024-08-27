// Copyright OxidOS Automotive 2024.

use parse::Component;
use quote::quote;
use std::{collections::HashMap, error::Error, rc::Rc};

/// Sort topologically a dependency graph with instances as nodes, given the stack for the
/// traversed nodes containing the "roots" of the graph. The [`topological_sort`] function
/// returns a sorted vector with the nodes' initialization code.
pub fn topological_sort(
    stack: &mut Vec<Rc<dyn parse::Component>>,
) -> Result<Vec<proc_macro2::TokenStream>, Box<dyn Error>> {
    // Vector of visited identifier nodes for the dependency graph.
    let mut visited: HashMap<String, bool> = HashMap::new();

    // List of the nodes sorted topologically.
    let mut nodes: Vec<Rc<dyn Component>> = Vec::new();
    // Loop while the stack still has untraversed nodes.
    'outer: while !stack.is_empty() {
        // SAFETY: Stack is not empty, so it contains at least one element, that the
        // last method could return.
        let front = stack.last().unwrap();

        // Mark the node as visited.
        visited.insert(front.ident()?.to_string(), true);

        // Iterate over the children of the current node.
        if let Some(dependencies) = front.dependencies() {
            for dep in dependencies {
                // Add the node to the stack if it hasn't been visited.
                if let None | Some(false) = visited.get(&dep.ident()?) {
                    stack.push(dep);
                    continue 'outer;
                }
            }
        }

        // If the code got through this part, the node either doesn't have any dependencies,
        // either all dependencies are visited. Can be added now to the list of topologically
        // sorted nodes.
        //
        // SAFETY: If the call from the `last` function didn't panic, neither will `pop`.
        nodes.push(stack.pop().unwrap());
    }

    Ok(nodes
        .into_iter()
        .flat_map(|node| {
            // This is not compiling yet. pushed it for Darius
            node.init_expr().map(|initialization| {
                let identifier: proc_macro2::TokenStream = node.ident().unwrap().parse().unwrap();
                let before_init = node.before_init().unwrap_or_default();
                let after_init = node.after_init().unwrap_or_default();

                quote! {
                    #before_init
                    let #identifier = #initialization;
                    #after_init
                }
            })
        })
        .collect::<Vec<_>>())
}
