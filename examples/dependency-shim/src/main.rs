use serde::{Deserialize, Serialize};
use uuid::Uuid;

who::warn!(
    dependency("uuid").compare(">=1.25.0"),
    "uuid-rs/uuid#902 added uuid::serde::bytes in uuid 1.25.0; review whether this shim can be replaced"
);

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
struct CompactRecord(#[serde(with = "uuid::serde::compact")] Uuid);

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
struct BytesRecord(#[serde(with = "uuid_serde_bytes")] Uuid);

mod uuid_serde_bytes {
    use serde::de::{self, SeqAccess, Visitor};
    use serde::{Deserializer, Serializer};
    use std::fmt;
    use uuid::Uuid;

    pub fn serialize<S>(value: &Uuid, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_bytes(value.as_bytes())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Uuid, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct UuidBytesVisitor;

        impl<'de> Visitor<'de> for UuidBytesVisitor {
            type Value = Uuid;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("16 bytes containing a UUID")
            }

            fn visit_bytes<E>(self, value: &[u8]) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                let bytes: [u8; 16] = value
                    .try_into()
                    .map_err(|_| E::invalid_length(value.len(), &self))?;
                Ok(Uuid::from_bytes(bytes))
            }

            fn visit_byte_buf<E>(self, value: Vec<u8>) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                self.visit_bytes(&value)
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let mut bytes = [0_u8; 16];

                for (index, slot) in bytes.iter_mut().enumerate() {
                    *slot = seq
                        .next_element()?
                        .ok_or_else(|| de::Error::invalid_length(index, &self))?;
                }

                if seq.next_element::<u8>()?.is_some() {
                    return Err(de::Error::invalid_length(17, &self));
                }

                Ok(Uuid::from_bytes(bytes))
            }
        }

        deserializer.deserialize_bytes(UuidBytesVisitor)
    }
}

fn main() {
    let id = Uuid::parse_str("77459ff3-c409-4d6a-a0a3-8b51ba3b56cf").expect("valid UUID");
    let compact = CompactRecord(id);
    let shimmed = BytesRecord(id);

    let compact_cbor = serde_cbor::to_vec(&compact).expect("compact encoding");
    let shimmed_cbor = serde_cbor::to_vec(&shimmed).expect("shim encoding");
    let decoded: BytesRecord = serde_cbor::from_slice(&shimmed_cbor).expect("shim decoding");

    assert_eq!(decoded, shimmed);
    assert_eq!(compact_cbor.len(), 32);
    assert_eq!(shimmed_cbor.len(), 17);

    println!(
        "uuid::serde::compact => {} bytes, shimmed byte-string encoding => {} bytes",
        compact_cbor.len(),
        shimmed_cbor.len()
    );
}
