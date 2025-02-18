//! **Warning:** The contents of `test_fuzz::utils` are provided for convenience and may be removed
//! in future versions of `test-fuzz`.

/// Skip values of type `$ty` when serializing. Initialize values of type `$ty` with `$expr` when
/// deserializing.
#[macro_export]
macro_rules! dont_care {
    ($ty:path, $expr:expr) => {
        impl serde::Serialize for $ty {
            fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                ().serialize(serializer)
            }
        }

        impl<'de> serde::Deserialize<'de> for $ty {
            fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                <()>::deserialize(deserializer).map(|_| $expr)
            }
        }
    };
    ($ty:path) => {
        $crate::dont_care!($ty, $ty);
    };
}

/// Wrap `<$ty as ToOwned>::Owned` in a type `$ident` and implement `From` and `test_fuzz::Into`
/// for `$ident` so that `convert = "&$ty, $ident"` can be used.
#[macro_export]
macro_rules! leak {
    ($ty:ty, $ident:ident) => {
        #[derive(Clone, std::fmt::Debug, serde::Deserialize, serde::Serialize)]
        struct $ident(<$ty as ToOwned>::Owned);

        impl From<&$ty> for $ident {
            fn from(ty: &$ty) -> Self {
                Self(ty.to_owned())
            }
        }

        impl test_fuzz::Into<&$ty> for $ident {
            fn into(self) -> &'static $ty {
                Box::leak(Box::new(self.0))
            }
        }
    };
}

pub mod serde_ref {
    pub use super::deserialize_ref as deserialize;
    pub use super::serialize_ref as serialize;
}

/// `serialize_ref` functions similar to `leak!`, but it is meant to be used with Serde's
/// [`serialize_with`](https://serde.rs/field-attrs.html#serialize_with) field attribute.
#[inline]
pub fn serialize_ref<'a, S, T>(x: &&'a T, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
    T: serde::Serialize,
{
    use serde_combinators::{RefF, SerializeWith, Type};

    <RefF<'a, Type<T>> as SerializeWith>::serialize(x, serializer)
}

/// `deserialize_ref` functions similar to `leak!`, but it is meant to be used with Serde's
/// [`deserialize_with`](https://serde.rs/field-attrs.html#deserialize_with) field attribute.
#[inline]
pub fn deserialize_ref<'de, D, T>(deserializer: D) -> Result<&'static T, D::Error>
where
    D: serde::Deserializer<'de>,
    T: serde::de::DeserializeOwned + std::fmt::Debug,
{
    use serde_combinators::{DeserializeWith, RefF, Type};

    <RefF<'static, Type<T>> as DeserializeWith>::deserialize(deserializer)
}

pub mod serde_ref_mut {
    pub use super::deserialize_ref_mut as deserialize;
    pub use super::serialize_ref_mut as serialize;
}

/// `serialize_ref_mut` is similar to `serialize_ref`, except it operates on a mutable reference
/// instead of an immutable one.
pub fn serialize_ref_mut<'a, S, T>(x: &&'a mut T, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
    T: serde::Serialize,
{
    use serde_combinators::{RefMutF, SerializeWith, Type};

    <RefMutF<'a, Type<T>> as SerializeWith>::serialize(x, serializer)
}

/// `deserialize_ref_mut` is similar to `deserialize_ref`, except it operates on a mutable reference
/// instead of an immutable one.
pub fn deserialize_ref_mut<'de, D, T>(deserializer: D) -> Result<&'static mut T, D::Error>
where
    D: serde::Deserializer<'de>,
    T: serde::de::DeserializeOwned + std::fmt::Debug,
{
    use serde_combinators::{DeserializeWith, RefMutF, Type};

    <RefMutF<'static, Type<T>> as DeserializeWith>::deserialize(deserializer)
}
