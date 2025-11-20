pub mod aes;
pub mod datetime;
pub mod messaging;

pub fn mask_identity(identity: String) -> String {
    identity
        .char_indices()
        .map(|(index, char)| if index == 0 { char } else { '·' })
        .collect()
}
