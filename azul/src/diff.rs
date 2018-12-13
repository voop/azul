//! Compares and diffs two DOM trees - necessary for tracking stateful events
//! such as user focus and scroll states across frames
use {
    dom::DomHash,
    id_tree::{NodeId, Arena},
};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct DomRange {
    pub start: DomNodeInfo,
    pub end: DomNodeInfo,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct DomNodeInfo {
    pub hash: DomHash,
    pub id: NodeId,
}

impl DomRange {
    /// Is `other` a subtree of `self`? - Assumes that the DOM was
    /// constructed in a linear order, i.e. the child being within
    /// the parents start / end bounds
    pub fn contains(&self, other: &Self) -> bool {
        other.start.id.index() >= self.start.id.index() &&
        other.end.id.index() <= self.end.id.index()
    }

    /// Compares two DOM ranges without looking at the DOM hashes
    pub fn equals_range(&self, other: &Self) -> bool {
        other.start == self.start &&
        other.end == self.end
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
pub struct DomDiff {
    /// What nodes have been added
    added_nodes: Vec<DomRange>,
    /// What nodes have been removed
    removed_nodes: Vec<DomRange>,
    /// Which nodes were shifted (from old position to new position)
    shifted_nodes: Vec<(DomRange, DomRange)>,
}

pub(crate) fn diff_dom_tree(old: &Arena<DomHash>, new: &Arena<DomHash>) -> DomDiff {

    use ui_solver::get_non_leaf_nodes_sorted_by_depth;

    let old_non_leaf_nodes = get_non_leaf_nodes_sorted_by_depth(&old.node_layout);
    let new_non_leaf_nodes = get_non_leaf_nodes_sorted_by_depth(&new.node_layout);

    let mut current_idx;
    let mut current_depth;

    for (parent_idx, (old_depth, old_parent_id)) in old_non_leaf_nodes.iter().enumerate() {
        current_idx = parent_idx;
        current_depth = old_depth;
        if let Some((new_depth, new_parent_id)) = new_non_leaf_nodes.get(current_idx) {
            if new_depth == old_depth {
                if new_parent_id == old_parent_id {
                    if old.node_data[*old_parent_id] == new.node_data[*new_parent_id] {
                        // Old and new parent are at the same DOM position and have the same content
                    } else {
                        // Old and new parent are at the same DOM position, but have different content
                    }
                } else {
                    // new and old parent have the same position, but may not have the same content
                }
            } else {
                // While the parents aren't exhausted yet, the new depth is not the old depth
            }
        } else {
            // The old DOM still has at least one parent left, but the new DOM has no more parents
        }
    }

    DomDiff::default()
}

/// Determines if the focus should be shifted, and if yes, to what node
pub(crate) fn shift_focus(diff: &DomDiff, old_focus: NodeId) -> Option<NodeId> {
    // TODO
    None
}

// how should the DOM diff look like?
#[test]
fn test_ui_hierarchy_diff() {
    // This is the most common kind of diff: Two arenas have the same content,
    // but a different hierarchy
    use id_tree::{Node, Arena, NodeHierarchy, NodeDataContainer};

    // Construct the old DOM:
    let old_node_content = vec![
        DomHash(0),
        DomHash(1),
        DomHash(2),
        DomHash(3),
        DomHash(4),
    ];

    // 0        - Content(0)
    // |- 1     - Content(1)
    // |- 2     - Content(2)
    //    |- 3  - Content(3)
    //    |- 4  - Content(4)
    let old_node_hierarchy = vec![
        // 0
        Node {
            parent: None,
            first_child: Some(NodeId::new(1)),
            last_child: Some(NodeId::new(2)),
            next_sibling: None,
            previous_sibling: None,
        },
        // 1
        Node {
            parent: Some(NodeId::new(0)),
            first_child: None,
            last_child: None,
            next_sibling: Some(NodeId::new(2)),
            previous_sibling: None,
        },
        // 2
        Node {
            parent: Some(NodeId::new(0)),
            first_child: Some(NodeId::new(3)),
            last_child: Some(NodeId::new(4)),
            next_sibling: None,
            previous_sibling: Some(NodeId::new(1)),
        },
        // 3
        Node {
            parent: Some(NodeId::new(2)),
            first_child: None,
            last_child: None,
            next_sibling: Some(NodeId::new(4)),
            previous_sibling: None,
        },
        // 4
        Node {
            parent: Some(NodeId::new(2)),
            first_child: None,
            last_child: None,
            next_sibling: None,
            previous_sibling: Some(NodeId::new(3)),
        }
    ];

    // 0        - Content(0)
    // |- 1     - Content(2)
    //    |- 2  - Content(3)
    //    |- 3  - Content(4)
    // |- 4     - Content(1)
    let new_node_content = vec![
        DomHash(0),
        DomHash(2),
        DomHash(3),
        DomHash(4),
        DomHash(1),
    ];

    let new_node_hierarchy = vec![
        // 0
        Node {
            parent: None,
            first_child: Some(NodeId::new(1)),
            last_child: Some(NodeId::new(4)),
            next_sibling: None,
            previous_sibling: None,
        },
        // 1
        Node {
            parent: Some(NodeId::new(0)),
            first_child: Some(NodeId::new(2)),
            last_child: Some(NodeId::new(3)),
            next_sibling: Some(NodeId::new(4)),
            previous_sibling: None,
        },
        // 2
        Node {
            parent: Some(NodeId::new(1)),
            first_child: None,
            last_child: None,
            next_sibling: Some(NodeId::new(3)),
            previous_sibling: None,
        },
        // 3
        Node {
            parent: Some(NodeId::new(1)),
            first_child: None,
            last_child: None,
            next_sibling: None,
            previous_sibling: Some(NodeId::new(2)),
        },
        // 4
        Node {
            parent: Some(NodeId::new(0)),
            first_child: None,
            last_child: None,
            next_sibling: None,
            previous_sibling: Some(NodeId::new(1)),
        },
    ];


    // Element with - Content(1) has been shifted from
    // DOM position 1 to position 4.

    let expected_diff = DomDiff {
        shifted_nodes: vec![
            (DomRange {
                // old position
                start: DomNodeInfo { id: NodeId::new(1), hash: DomHash(1), },
                end: DomNodeInfo { id: NodeId::new(1), hash: DomHash(1), },
            },
            DomRange {
                // new position
                start: DomNodeInfo { id: NodeId::new(4), hash: DomHash(1), },
                end: DomNodeInfo { id: NodeId::new(4), hash: DomHash(1), },
            })
        ],
        .. Default::default()
    };

    let old_arena = Arena {
        node_layout: NodeHierarchy { internal: old_node_hierarchy },
        node_data: NodeDataContainer { internal: old_node_content },
    };
    let new_arena = Arena {
        node_layout: NodeHierarchy { internal: new_node_hierarchy },
        node_data: NodeDataContainer { internal: new_node_content },
    };

    assert_eq!(diff_dom_tree(&old_arena, &new_arena), expected_diff);

    // If the focus was previously on NodeId 1, shift the focus on NodeId 4
    assert_eq!(shift_focus(&expected_diff, NodeId::new(1)), Some(NodeId::new(4)));
    // Because of the shift, if the element 3 was focused, now the element 2 has focus
    // (because the node and the subtree weren't removed)
    assert_eq!(shift_focus(&expected_diff, NodeId::new(3)), Some(NodeId::new(2)));
    assert_eq!(shift_focus(&expected_diff, NodeId::new(2)), Some(NodeId::new(1)));

    // Element 0 has not experienced a shift, since the shift was only in the sub-tree
    assert_eq!(shift_focus(&expected_diff, NodeId::new(0)), None);

}