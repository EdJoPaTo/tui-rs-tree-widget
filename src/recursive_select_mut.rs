use crate::GenericTreeItem;

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
