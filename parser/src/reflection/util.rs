use super::{TypeCollection, TypeInfo};

#[inline]
pub fn prefix_pattern<const N: usize, const M: usize>(
    pattern: [u8; N],
    value: u8
) -> [u8; M] {
    let mut new_pattern = [0; M];
    new_pattern[0] = value;
    new_pattern[1..M].copy_from_slice(&pattern);
    new_pattern
}

impl TypeCollection {

    pub(super) fn get_inner_type(&self, type_entry: &TypeInfo) -> &TypeInfo {
        type_entry.inner_type.as_ref()
            .and_then(|t| self.get_type(*t))
            .map(|t| self.resolve_typedef(t))
            .expect("invalid inner type")
    }

    pub(super) fn get_inner_type_opt(&self, type_entry: &TypeInfo) -> Option<&TypeInfo> {
        type_entry.inner_type.as_ref()
            .and_then(|t| self.get_type(*t))
            .map(|t| self.resolve_typedef(t))
    }

}
