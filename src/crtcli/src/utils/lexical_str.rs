use std::cmp::Ordering;

pub fn iterate_ascii_only_alnum(s: &'_ [u8]) -> impl DoubleEndedIterator<Item = char> + '_ {
    s.iter().flat_map(|x| {
        if x.is_ascii_alphanumeric() {
            Some(*x as char)
        } else {
            None
        }
    })
}

pub fn ascii_alnum_cmp(s1: &[u8], s2: &[u8]) -> Ordering {
    let mut iter1 = iterate_ascii_only_alnum(s1);
    let mut iter2 = iterate_ascii_only_alnum(s2);

    loop {
        match (iter1.next(), iter2.next()) {
            (Some(lhs), Some(rhs)) => {
                if lhs != rhs {
                    return lhs.cmp(&rhs);
                }
            }
            (Some(_), None) => return Ordering::Greater,
            (None, Some(_)) => return Ordering::Less,
            _ => return s1.cmp(s2),
        }
    }
}
