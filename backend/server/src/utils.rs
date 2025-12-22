pub mod aes;
pub mod csv;
pub mod datetime;
pub mod session;

pub fn mask_identity(identity: String) -> String {
    identity
        .char_indices()
        .map(|(index, char)| if index == 0 { char } else { '·' })
        .collect()
}
