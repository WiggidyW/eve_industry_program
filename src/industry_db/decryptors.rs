use super::*;

pub fn find_matching_decryptor(
    base: Item,
    product: Item,
    decryptor_match: u32,
) -> Option<(Item, f64)> {
    for (decryptor, p_mult) in DECRYPTORS {
        if decryptor.type_id == decryptor_match
            && base.runs + decryptor.runs == product.runs
            && base.me + decryptor.me == product.me
            && base.te + decryptor.te == product.te
        {
            return Some((decryptor.into_non_blueprint(), p_mult));
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
