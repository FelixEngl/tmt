//Copyright 2024 Felix Engl
//
//Licensed under the Apache License, Version 2.0 (the "License");
//you may not use this file except in compliance with the License.
//You may obtain a copy of the License at
//
//    http://www.apache.org/licenses/LICENSE-2.0
//
//Unless required by applicable law or agreed to in writing, software
//distributed under the License is distributed on an "AS IS" BASIS,
//WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
//See the License for the specific language governing permissions and
//limitations under the License.

#![allow(dead_code)]

use std::fmt::{Display, Formatter};
use evalexpr::{Node, Operator};
use itertools::Itertools;
use strum::EnumIs;


/// Walks a node from left to right
pub(crate) fn walk_left_to_right(node: &Node) -> NodeContainer {
    fn walk_left_to_right_(node: &Node, is_root: bool) -> NodeContainer {
        let children = node.children();
        match children.len() {
            0 => {
                NodeContainer::Leaf(node, is_root)
            }
            1 => {
                NodeContainer::Single(node, walk_left_to_right_(&children[0], false).into(), is_root)
            }
            2 => {
                NodeContainer::Expr(
                    walk_left_to_right_(&children[0], false).into(),
                    node,
                    walk_left_to_right_(&children[1], false).into(),
                    is_root
                )
            }
            _ => {
                NodeContainer::Special(node, node.children().iter().map(
                    |value| walk_left_to_right_(value, false)
                ).collect_vec(), is_root)
            }
        }
    }
    walk_left_to_right_(node, true)
}

/// A node container for walking
#[derive(Debug, Clone)]
#[derive(EnumIs)]
pub(crate) enum NodeContainer<'a> {
    Leaf(&'a Node, bool),
    Single(&'a Node, Box<NodeContainer<'a>>, bool),
    Expr(Box<NodeContainer<'a>>, &'a Node, Box<NodeContainer<'a>>, bool),
    Special(&'a Node, Vec<NodeContainer<'a>>, bool)
}

impl<'a> NodeContainer<'a> {

    fn origin(&self) -> &'a Node {
        match self {
            NodeContainer::Leaf(value, _) => {*value}
            NodeContainer::Single(value, _, _) => {*value}
            NodeContainer::Expr(_, value, _, _) => {*value}
            NodeContainer::Special(value, _, _) => {*value}
        }
    }
}

impl Display for NodeContainer<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            NodeContainer::Leaf(value, _) => {
                write!(f, "{}", format!("{}", value).trim())
            }
            NodeContainer::Single(value1, value2, is_root) => {
                if *is_root {
                    write!(f, "{}", value2.as_ref())
                } else {
                    match value1.operator() {
                        Operator::RootNode => {
                            if value2.is_expr() {
                                write!(f, "({})", value2.as_ref())
                            } else {
                                write!(f, "{}", value2.as_ref())
                            }
                        }
                        _ => {
                            write!(f, "{}{}", format!("{}", value1.operator()).trim(), value2.as_ref())
                        }
                    }
                }
            }
            NodeContainer::Expr(value1, value2, value3, _) => {
                write!(f, "{} {} {}", value1, format!("{}", value2.operator()).trim(), value3)
            }
            NodeContainer::Special(value1, value2, _) => {
                match value1.operator() {
                    Operator::Tuple => {
                        write!(f, "({})", value2.iter().join(", "))
                    }
                    Operator::Chain => {
                        write!(f, "{}", value2.iter().join("; "))
                    }
                    _ => write!(f, "[!{value1}]")
                }

            }
        }
    }
}



