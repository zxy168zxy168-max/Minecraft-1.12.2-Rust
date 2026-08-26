use crate::net::minecraft::nbt::NBTBase::NBTBase;
use crate::net::minecraft::nbt::NBTTagCompound::NBTTagCompound;

/// MCP 1.12.2 `NBTUtil.areNBTEquals` subset used by villager recipes.
/// When `compareListTag` is false, list order is ignored exactly as vanilla's
/// recursive matcher; compounds require every key in the expected value to be
/// present in the candidate but may contain additional keys.
pub fn areNBTEquals(
    expected: &NBTTagCompound,
    candidate: &NBTTagCompound,
    compareListTag: bool,
) -> bool {
    expected
        .getKeySet()
        .all(|key| match (expected.getTag(key), candidate.getTag(key)) {
            (Some(left), Some(right)) => areTagEquals(left, right, compareListTag),
            _ => false,
        })
}

fn areTagEquals(expected: &NBTBase, candidate: &NBTBase, compareListTag: bool) -> bool {
    match (expected, candidate) {
        (NBTBase::Compound(left), NBTBase::Compound(right)) => {
            areNBTEquals(left, right, compareListTag)
        }
        (NBTBase::List(left), NBTBase::List(right)) if !compareListTag => {
            if left.tagCount() == 0 {
                return right.tagCount() == 0;
            }
            if left.tagCount() > right.tagCount() {
                return false;
            }
            (0..left.tagCount()).all(|i| {
                let Some(wanted) = left.tags().get(i) else {
                    return false;
                };
                (0..right.tagCount()).any(|j| {
                    right
                        .tags()
                        .get(j)
                        .is_some_and(|got| areTagEquals(wanted, got, false))
                })
            })
        }
        _ => expected == candidate,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn expected_compound_is_a_subset() {
        let mut expected = NBTTagCompound::new();
        expected.setInteger("a", 1);
        let mut candidate = expected.clone();
        candidate.setString("extra", "allowed");
        assert!(areNBTEquals(&expected, &candidate, false));
        assert!(!areNBTEquals(&candidate, &expected, false));
    }
}
