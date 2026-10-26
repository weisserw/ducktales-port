//! Versioned game snapshots. Arrays serialize independently of serde's fixed lengths.
pub mod array {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    pub fn serialize<T: Serialize, S: Serializer, const N: usize>(
        value: &[T; N],
        s: S,
    ) -> Result<S::Ok, S::Error> {
        value.as_slice().serialize(s)
    }
    pub fn deserialize<'de, T: Deserialize<'de>, D: Deserializer<'de>, const N: usize>(
        d: D,
    ) -> Result<[T; N], D::Error> {
        let value = Vec::<T>::deserialize(d)?;
        value.try_into().map_err(|v: Vec<T>| {
            serde::de::Error::custom(format!("expected {N} entries, got {}", v.len()))
        })
    }
}
