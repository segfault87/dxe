use dxe_types::UserId;

pub trait Identity {
    fn provider() -> &'static str;

    fn user_id(&self) -> &UserId;
}
