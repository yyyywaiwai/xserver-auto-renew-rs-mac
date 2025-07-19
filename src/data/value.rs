#[macro_export]
macro_rules! db_accessors {
    ($base:ident, $key:expr, $ty:ty $(,)?) => {
        ::paste::paste! {
            /// Setter：`set_<base>(value)`
            #[allow(dead_code)]
            pub fn [<set_ $base>](value: &$ty)
            where
                $ty: ::bincode::Encode,
            {
                $crate::data::put($key, value);
            }

            #[allow(dead_code)]
            /// Getter：`get_<base>() -> Option<_>`
            pub fn [<get_ $base>]() -> Option<$ty>
            where
                $ty: ::bincode::Decode<()>,
            {
                $crate::data::get($key)
            }

            #[allow(dead_code)]
            /// Remover：`remove_<base>() -> bool`
            pub fn [<remove_ $base>]() -> bool {
                $crate::data::remove($key)
            }
        }
    };
}

db_accessors!(account, b"account_v1", crate::client::Account);
db_accessors!(ua, b"user_agent_v1", String);
db_accessors!(cookie, b"cookie_v1", String);
db_accessors!(webhook, b"webhook_v1", String);
db_accessors!(two_captcha_key, b"two_captcha_key", String);
