use crate::GenericTreeItem;

#[must_use]
pub fn get_item<'root, Item: GenericTreeItem>(
    root: &'root [Item],
    identifier: &[Item::Identifier],
) -> Option<&'root Item> {
    let mut identifier = identifier.iter();
    let initial_identifier = identifier.next()?;
    let mut current = root
        .iter()
        .find(|item| item.identifier() == initial_identifier)?;
    for identifier in identifier {
        current = current.child_direct(identifier)?;
    }
    Some(current)
}

#[must_use]
pub fn get_item_mut<'root, Item>(
    root: &'root mut [Item],
    identifier: &[<Item as GenericTreeItem>::Identifier],
) -> Option<&'root mut Item>
where
    Item: GenericTreeItem + RecursiveSelectMut<Identifier = <Item as GenericTreeItem>::Identifier>,
{
    let mut identifier = identifier.iter();
    let initial_identifier = identifier.next()?;
    let mut current = root
        .iter_mut()
        .find(|item| item.identifier() == initial_identifier)?;
    for identifier in identifier {
        current = current.child_direct_mut(identifier)?;
    }
    Some(current)
}

pub trait RecursiveSelect {
    type Identifier;

    #[must_use]
    fn child_direct<'root>(&'root self, identifier: &Self::Identifier) -> Option<&'root Self>;

    #[must_use]
    fn child_deep<'root>(&'root self, identifier: &[Self::Identifier]) -> Option<&'root Self> {
        let mut current = self;
        for identifier in identifier {
            current = current.child_direct(identifier)?;
        }
        Some(current)
    }
}

#[allow(clippy::module_name_repetitions)]
pub trait RecursiveSelectMut {
    type Identifier;

    #[must_use]
    fn child_direct_mut<'root>(
        &'root mut self,
        identifier: &Self::Identifier,
    ) -> Option<&'root mut Self>;

    #[must_use]
    fn child_deep_mut<'root>(
        &'root mut self,
        identifier: &[Self::Identifier],
    ) -> Option<&'root mut Self> {
        let mut current = self;
        for identifier in identifier {
            current = current.child_direct_mut(identifier)?;
        }
        Some(current)
    }
}

impl<Item: GenericTreeItem> RecursiveSelect for Item {
    type Identifier = Item::Identifier;

    fn child_direct<'root>(&'root self, identifier: &Self::Identifier) -> Option<&'root Self> {
        self.children()
            .iter()
            .find(|item| item.identifier() == identifier)
    }
}
