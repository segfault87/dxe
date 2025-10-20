use dxe_identity::Identity;

pub enum DxeIdentity {
    Biztalk,
}

impl Identity for DxeIdentity {
    fn provider() -> &'static str {
        "BIZTALK"
    }
}
