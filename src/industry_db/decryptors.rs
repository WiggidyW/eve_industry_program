use super::*;

pub fn find_matching_decryptor(
    base: Item,
    product: Item,
    decryptor_match: TypeId,
) -> Option<(Item, f64)> {
    for decryptor in DECRYPTORS {
        if decryptor.0.type_id == decryptor_match
            && base.runs + decryptor.0.runs == product.runs
            && base.me + decryptor.0.me == product.me
            && base.te + decryptor.0.te == product.te
        {
            return Some(decryptor);
        }
    }
    None
}

pub fn closest_pmult(base: f64, new: f64) -> (Item, f64) {
    let mut closest_diff = (new - base).abs();
    let mut closest = (Item::null(), 1.0);
    for decryptor in DECRYPTORS {
        let diff = (base * decryptor.1 - base).abs();
        if diff < closest_diff {
            closest_diff = diff;
            closest = decryptor;
        }
    }
    closest
}
