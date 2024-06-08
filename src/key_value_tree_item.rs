use crate::{Node, TreeData};

/// Tree Item implementation for key-value data structures like JSON.
///
/// Sadly this can not be implemented by third party crates as traits can only be implemented
/// within the crate that defines the trait or the crate that defines the struct.
/// The newtype pattern doesnt work here either as `get_value` returns `&Self` resulting in
/// referencing local data.
pub trait KeyValueTreeItem {
    type Key: Clone + PartialEq + Eq + core::hash::Hash;

    #[must_use]
    fn keys_below(&self) -> Vec<Self::Key>;

    #[must_use]
    fn has_children(&self) -> bool {
        !self.keys_below().is_empty()
    }

    #[must_use]
    fn get_value(&self, key: &Self::Key) -> Option<&Self>;

    #[must_use]
    fn get_value_deep<'root>(&'root self, keys: &[Self::Key]) -> Option<&'root Self> {
        let mut current = self;
        for key in keys {
            current = current.get_value(key)?;
        }
        Some(current)
    }

    #[must_use]
    fn get_children(&self) -> Vec<(Self::Key, &Self)> {
        self.keys_below()
            .into_iter()
            .map(|key| {
                let value = self
                    .get_value(&key)
                    .expect("value should exist when the key exist");
                (key, value)
            })
            .collect()
    }

    #[must_use]
    fn height(&self) -> usize;

    fn render(
        &self,
        key: Option<&Self::Key>,
        area: ratatui::layout::Rect,
        buffer: &mut ratatui::buffer::Buffer,
    );
}

impl<Item: KeyValueTreeItem> TreeData for Item {
    type Identifier = Vec<Item::Key>;

    fn get_nodes(
        &self,
        open_identifiers: &std::collections::HashSet<Self::Identifier>,
    ) -> Vec<crate::Node<Self::Identifier>> {
        if self.has_children() {
            flatten(open_identifiers, self.get_children(), &[])
        } else {
            vec![Node {
                depth: 0,
                has_children: false,
                height: self.height(),
                identifier: vec![],
            }]
        }
    }

    fn render(
        &self,
        identifier: &Self::Identifier,
        area: ratatui::layout::Rect,
        buffer: &mut ratatui::buffer::Buffer,
    ) {
        if let Some(value) = self.get_value_deep(identifier) {
            let parent = identifier.last();
            value.render(parent, area, buffer);
        }
    }
}

#[must_use]
fn flatten<Item: KeyValueTreeItem>(
    open_identifiers: &std::collections::HashSet<Vec<Item::Key>>,
    values: Vec<(Item::Key, &Item)>,
    current: &[Item::Key],
) -> Vec<Node<Vec<Item::Key>>> {
    let depth = current.len();
    let mut result = Vec::new();
    for (key, value) in values {
        let mut child_identifier = current.to_vec();
        child_identifier.push(key);

        let child_result = open_identifiers
            .contains(&child_identifier)
            .then(|| flatten(open_identifiers, value.get_children(), &child_identifier));

        result.push(Node {
            depth,
            has_children: value.has_children(),
            height: value.height(),
            identifier: child_identifier,
        });

        if let Some(mut child_result) = child_result {
            result.append(&mut child_result);
        }
    }
    result
}
