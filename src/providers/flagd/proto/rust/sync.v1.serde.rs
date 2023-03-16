// @generated
impl serde::Serialize for FetchAllFlagsRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let len = 0;
        let struct_ser = serializer.serialize_struct("sync.v1.FetchAllFlagsRequest", len)?;
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for FetchAllFlagsRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                            Err(serde::de::Error::unknown_field(value, FIELDS))
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = FetchAllFlagsRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct sync.v1.FetchAllFlagsRequest")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<FetchAllFlagsRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                while map.next_key::<GeneratedField>()?.is_some() {
                    let _ = map.next_value::<serde::de::IgnoredAny>()?;
                }
                Ok(FetchAllFlagsRequest {
                })
            }
        }
        deserializer.deserialize_struct("sync.v1.FetchAllFlagsRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for FetchAllFlagsResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.flag_configuration.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("sync.v1.FetchAllFlagsResponse", len)?;
        if !self.flag_configuration.is_empty() {
            struct_ser.serialize_field("flagConfiguration", &self.flag_configuration)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for FetchAllFlagsResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "flag_configuration",
            "flagConfiguration",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            FlagConfiguration,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "flagConfiguration" | "flag_configuration" => Ok(GeneratedField::FlagConfiguration),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = FetchAllFlagsResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct sync.v1.FetchAllFlagsResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<FetchAllFlagsResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut flag_configuration__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::FlagConfiguration => {
                            if flag_configuration__.is_some() {
                                return Err(serde::de::Error::duplicate_field("flagConfiguration"));
                            }
                            flag_configuration__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(FetchAllFlagsResponse {
                    flag_configuration: flag_configuration__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("sync.v1.FetchAllFlagsResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for SyncFlagsRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.provider_id.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("sync.v1.SyncFlagsRequest", len)?;
        if !self.provider_id.is_empty() {
            struct_ser.serialize_field("providerId", &self.provider_id)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for SyncFlagsRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "provider_id",
            "providerId",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ProviderId,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "providerId" | "provider_id" => Ok(GeneratedField::ProviderId),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = SyncFlagsRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct sync.v1.SyncFlagsRequest")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<SyncFlagsRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut provider_id__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::ProviderId => {
                            if provider_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("providerId"));
                            }
                            provider_id__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(SyncFlagsRequest {
                    provider_id: provider_id__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("sync.v1.SyncFlagsRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for SyncFlagsResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.flag_configuration.is_empty() {
            len += 1;
        }
        if self.state != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("sync.v1.SyncFlagsResponse", len)?;
        if !self.flag_configuration.is_empty() {
            struct_ser.serialize_field("flagConfiguration", &self.flag_configuration)?;
        }
        if self.state != 0 {
            let v = SyncState::from_i32(self.state)
                .ok_or_else(|| serde::ser::Error::custom(format!("Invalid variant {}", self.state)))?;
            struct_ser.serialize_field("state", &v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for SyncFlagsResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "flag_configuration",
            "flagConfiguration",
            "state",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            FlagConfiguration,
            State,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "flagConfiguration" | "flag_configuration" => Ok(GeneratedField::FlagConfiguration),
                            "state" => Ok(GeneratedField::State),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = SyncFlagsResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct sync.v1.SyncFlagsResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<SyncFlagsResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut flag_configuration__ = None;
                let mut state__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::FlagConfiguration => {
                            if flag_configuration__.is_some() {
                                return Err(serde::de::Error::duplicate_field("flagConfiguration"));
                            }
                            flag_configuration__ = Some(map.next_value()?);
                        }
                        GeneratedField::State => {
                            if state__.is_some() {
                                return Err(serde::de::Error::duplicate_field("state"));
                            }
                            state__ = Some(map.next_value::<SyncState>()? as i32);
                        }
                    }
                }
                Ok(SyncFlagsResponse {
                    flag_configuration: flag_configuration__.unwrap_or_default(),
                    state: state__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("sync.v1.SyncFlagsResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for SyncState {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let variant = match self {
            Self::Unspecified => "SYNC_STATE_UNSPECIFIED",
            Self::All => "SYNC_STATE_ALL",
            Self::Add => "SYNC_STATE_ADD",
            Self::Update => "SYNC_STATE_UPDATE",
            Self::Delete => "SYNC_STATE_DELETE",
            Self::Ping => "SYNC_STATE_PING",
        };
        serializer.serialize_str(variant)
    }
}
impl<'de> serde::Deserialize<'de> for SyncState {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "SYNC_STATE_UNSPECIFIED",
            "SYNC_STATE_ALL",
            "SYNC_STATE_ADD",
            "SYNC_STATE_UPDATE",
            "SYNC_STATE_DELETE",
            "SYNC_STATE_PING",
        ];

        struct GeneratedVisitor;

        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = SyncState;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(formatter, "expected one of: {:?}", &FIELDS)
            }

            fn visit_i64<E>(self, v: i64) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                use std::convert::TryFrom;
                i32::try_from(v)
                    .ok()
                    .and_then(SyncState::from_i32)
                    .ok_or_else(|| {
                        serde::de::Error::invalid_value(serde::de::Unexpected::Signed(v), &self)
                    })
            }

            fn visit_u64<E>(self, v: u64) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                use std::convert::TryFrom;
                i32::try_from(v)
                    .ok()
                    .and_then(SyncState::from_i32)
                    .ok_or_else(|| {
                        serde::de::Error::invalid_value(serde::de::Unexpected::Unsigned(v), &self)
                    })
            }

            fn visit_str<E>(self, value: &str) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match value {
                    "SYNC_STATE_UNSPECIFIED" => Ok(SyncState::Unspecified),
                    "SYNC_STATE_ALL" => Ok(SyncState::All),
                    "SYNC_STATE_ADD" => Ok(SyncState::Add),
                    "SYNC_STATE_UPDATE" => Ok(SyncState::Update),
                    "SYNC_STATE_DELETE" => Ok(SyncState::Delete),
                    "SYNC_STATE_PING" => Ok(SyncState::Ping),
                    _ => Err(serde::de::Error::unknown_variant(value, FIELDS)),
                }
            }
        }
        deserializer.deserialize_any(GeneratedVisitor)
    }
}
