// @generated
impl serde::Serialize for AnyFlag {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.reason.is_empty() {
            len += 1;
        }
        if !self.variant.is_empty() {
            len += 1;
        }
        if self.value.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("schema.v1.AnyFlag", len)?;
        if !self.reason.is_empty() {
            struct_ser.serialize_field("reason", &self.reason)?;
        }
        if !self.variant.is_empty() {
            struct_ser.serialize_field("variant", &self.variant)?;
        }
        if let Some(v) = self.value.as_ref() {
            match v {
                any_flag::Value::BoolValue(v) => {
                    struct_ser.serialize_field("boolValue", v)?;
                }
                any_flag::Value::StringValue(v) => {
                    struct_ser.serialize_field("stringValue", v)?;
                }
                any_flag::Value::DoubleValue(v) => {
                    struct_ser.serialize_field("doubleValue", v)?;
                }
                any_flag::Value::ObjectValue(v) => {
                    struct_ser.serialize_field("objectValue", v)?;
                }
            }
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for AnyFlag {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "reason",
            "variant",
            "bool_value",
            "boolValue",
            "string_value",
            "stringValue",
            "double_value",
            "doubleValue",
            "object_value",
            "objectValue",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Reason,
            Variant,
            BoolValue,
            StringValue,
            DoubleValue,
            ObjectValue,
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
                            "reason" => Ok(GeneratedField::Reason),
                            "variant" => Ok(GeneratedField::Variant),
                            "boolValue" | "bool_value" => Ok(GeneratedField::BoolValue),
                            "stringValue" | "string_value" => Ok(GeneratedField::StringValue),
                            "doubleValue" | "double_value" => Ok(GeneratedField::DoubleValue),
                            "objectValue" | "object_value" => Ok(GeneratedField::ObjectValue),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = AnyFlag;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct schema.v1.AnyFlag")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<AnyFlag, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut reason__ = None;
                let mut variant__ = None;
                let mut value__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Reason => {
                            if reason__.is_some() {
                                return Err(serde::de::Error::duplicate_field("reason"));
                            }
                            reason__ = Some(map.next_value()?);
                        }
                        GeneratedField::Variant => {
                            if variant__.is_some() {
                                return Err(serde::de::Error::duplicate_field("variant"));
                            }
                            variant__ = Some(map.next_value()?);
                        }
                        GeneratedField::BoolValue => {
                            if value__.is_some() {
                                return Err(serde::de::Error::duplicate_field("boolValue"));
                            }
                            value__ = map.next_value::<::std::option::Option<_>>()?.map(any_flag::Value::BoolValue);
                        }
                        GeneratedField::StringValue => {
                            if value__.is_some() {
                                return Err(serde::de::Error::duplicate_field("stringValue"));
                            }
                            value__ = map.next_value::<::std::option::Option<_>>()?.map(any_flag::Value::StringValue);
                        }
                        GeneratedField::DoubleValue => {
                            if value__.is_some() {
                                return Err(serde::de::Error::duplicate_field("doubleValue"));
                            }
                            value__ = map.next_value::<::std::option::Option<::pbjson::private::NumberDeserialize<_>>>()?.map(|x| any_flag::Value::DoubleValue(x.0));
                        }
                        GeneratedField::ObjectValue => {
                            if value__.is_some() {
                                return Err(serde::de::Error::duplicate_field("objectValue"));
                            }
                            value__ = map.next_value::<::std::option::Option<_>>()?.map(any_flag::Value::ObjectValue)
;
                        }
                    }
                }
                Ok(AnyFlag {
                    reason: reason__.unwrap_or_default(),
                    variant: variant__.unwrap_or_default(),
                    value: value__,
                })
            }
        }
        deserializer.deserialize_struct("schema.v1.AnyFlag", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for EventStreamRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let len = 0;
        let struct_ser = serializer.serialize_struct("schema.v1.EventStreamRequest", len)?;
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for EventStreamRequest {
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
            type Value = EventStreamRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct schema.v1.EventStreamRequest")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<EventStreamRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                while map.next_key::<GeneratedField>()?.is_some() {
                    let _ = map.next_value::<serde::de::IgnoredAny>()?;
                }
                Ok(EventStreamRequest {
                })
            }
        }
        deserializer.deserialize_struct("schema.v1.EventStreamRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for EventStreamResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.r#type.is_empty() {
            len += 1;
        }
        if self.data.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("schema.v1.EventStreamResponse", len)?;
        if !self.r#type.is_empty() {
            struct_ser.serialize_field("type", &self.r#type)?;
        }
        if let Some(v) = self.data.as_ref() {
            struct_ser.serialize_field("data", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for EventStreamResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "type",
            "data",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Type,
            Data,
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
                            "type" => Ok(GeneratedField::Type),
                            "data" => Ok(GeneratedField::Data),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = EventStreamResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct schema.v1.EventStreamResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<EventStreamResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut r#type__ = None;
                let mut data__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Type => {
                            if r#type__.is_some() {
                                return Err(serde::de::Error::duplicate_field("type"));
                            }
                            r#type__ = Some(map.next_value()?);
                        }
                        GeneratedField::Data => {
                            if data__.is_some() {
                                return Err(serde::de::Error::duplicate_field("data"));
                            }
                            data__ = map.next_value()?;
                        }
                    }
                }
                Ok(EventStreamResponse {
                    r#type: r#type__.unwrap_or_default(),
                    data: data__,
                })
            }
        }
        deserializer.deserialize_struct("schema.v1.EventStreamResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ResolveAllRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.context.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("schema.v1.ResolveAllRequest", len)?;
        if let Some(v) = self.context.as_ref() {
            struct_ser.serialize_field("context", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ResolveAllRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "context",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Context,
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
                            "context" => Ok(GeneratedField::Context),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ResolveAllRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct schema.v1.ResolveAllRequest")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<ResolveAllRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut context__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Context => {
                            if context__.is_some() {
                                return Err(serde::de::Error::duplicate_field("context"));
                            }
                            context__ = map.next_value()?;
                        }
                    }
                }
                Ok(ResolveAllRequest {
                    context: context__,
                })
            }
        }
        deserializer.deserialize_struct("schema.v1.ResolveAllRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ResolveAllResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.flags.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("schema.v1.ResolveAllResponse", len)?;
        if !self.flags.is_empty() {
            struct_ser.serialize_field("flags", &self.flags)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ResolveAllResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "flags",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Flags,
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
                            "flags" => Ok(GeneratedField::Flags),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ResolveAllResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct schema.v1.ResolveAllResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<ResolveAllResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut flags__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Flags => {
                            if flags__.is_some() {
                                return Err(serde::de::Error::duplicate_field("flags"));
                            }
                            flags__ = Some(
                                map.next_value::<std::collections::HashMap<_, _>>()?
                            );
                        }
                    }
                }
                Ok(ResolveAllResponse {
                    flags: flags__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("schema.v1.ResolveAllResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ResolveBooleanRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.flag_key.is_empty() {
            len += 1;
        }
        if self.context.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("schema.v1.ResolveBooleanRequest", len)?;
        if !self.flag_key.is_empty() {
            struct_ser.serialize_field("flagKey", &self.flag_key)?;
        }
        if let Some(v) = self.context.as_ref() {
            struct_ser.serialize_field("context", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ResolveBooleanRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "flag_key",
            "flagKey",
            "context",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            FlagKey,
            Context,
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
                            "flagKey" | "flag_key" => Ok(GeneratedField::FlagKey),
                            "context" => Ok(GeneratedField::Context),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ResolveBooleanRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct schema.v1.ResolveBooleanRequest")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<ResolveBooleanRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut flag_key__ = None;
                let mut context__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::FlagKey => {
                            if flag_key__.is_some() {
                                return Err(serde::de::Error::duplicate_field("flagKey"));
                            }
                            flag_key__ = Some(map.next_value()?);
                        }
                        GeneratedField::Context => {
                            if context__.is_some() {
                                return Err(serde::de::Error::duplicate_field("context"));
                            }
                            context__ = map.next_value()?;
                        }
                    }
                }
                Ok(ResolveBooleanRequest {
                    flag_key: flag_key__.unwrap_or_default(),
                    context: context__,
                })
            }
        }
        deserializer.deserialize_struct("schema.v1.ResolveBooleanRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ResolveBooleanResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.value {
            len += 1;
        }
        if !self.reason.is_empty() {
            len += 1;
        }
        if !self.variant.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("schema.v1.ResolveBooleanResponse", len)?;
        if self.value {
            struct_ser.serialize_field("value", &self.value)?;
        }
        if !self.reason.is_empty() {
            struct_ser.serialize_field("reason", &self.reason)?;
        }
        if !self.variant.is_empty() {
            struct_ser.serialize_field("variant", &self.variant)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ResolveBooleanResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "value",
            "reason",
            "variant",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Value,
            Reason,
            Variant,
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
                            "value" => Ok(GeneratedField::Value),
                            "reason" => Ok(GeneratedField::Reason),
                            "variant" => Ok(GeneratedField::Variant),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ResolveBooleanResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct schema.v1.ResolveBooleanResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<ResolveBooleanResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut value__ = None;
                let mut reason__ = None;
                let mut variant__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Value => {
                            if value__.is_some() {
                                return Err(serde::de::Error::duplicate_field("value"));
                            }
                            value__ = Some(map.next_value()?);
                        }
                        GeneratedField::Reason => {
                            if reason__.is_some() {
                                return Err(serde::de::Error::duplicate_field("reason"));
                            }
                            reason__ = Some(map.next_value()?);
                        }
                        GeneratedField::Variant => {
                            if variant__.is_some() {
                                return Err(serde::de::Error::duplicate_field("variant"));
                            }
                            variant__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(ResolveBooleanResponse {
                    value: value__.unwrap_or_default(),
                    reason: reason__.unwrap_or_default(),
                    variant: variant__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("schema.v1.ResolveBooleanResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ResolveFloatRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.flag_key.is_empty() {
            len += 1;
        }
        if self.context.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("schema.v1.ResolveFloatRequest", len)?;
        if !self.flag_key.is_empty() {
            struct_ser.serialize_field("flagKey", &self.flag_key)?;
        }
        if let Some(v) = self.context.as_ref() {
            struct_ser.serialize_field("context", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ResolveFloatRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "flag_key",
            "flagKey",
            "context",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            FlagKey,
            Context,
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
                            "flagKey" | "flag_key" => Ok(GeneratedField::FlagKey),
                            "context" => Ok(GeneratedField::Context),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ResolveFloatRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct schema.v1.ResolveFloatRequest")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<ResolveFloatRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut flag_key__ = None;
                let mut context__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::FlagKey => {
                            if flag_key__.is_some() {
                                return Err(serde::de::Error::duplicate_field("flagKey"));
                            }
                            flag_key__ = Some(map.next_value()?);
                        }
                        GeneratedField::Context => {
                            if context__.is_some() {
                                return Err(serde::de::Error::duplicate_field("context"));
                            }
                            context__ = map.next_value()?;
                        }
                    }
                }
                Ok(ResolveFloatRequest {
                    flag_key: flag_key__.unwrap_or_default(),
                    context: context__,
                })
            }
        }
        deserializer.deserialize_struct("schema.v1.ResolveFloatRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ResolveFloatResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.value != 0. {
            len += 1;
        }
        if !self.reason.is_empty() {
            len += 1;
        }
        if !self.variant.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("schema.v1.ResolveFloatResponse", len)?;
        if self.value != 0. {
            struct_ser.serialize_field("value", &self.value)?;
        }
        if !self.reason.is_empty() {
            struct_ser.serialize_field("reason", &self.reason)?;
        }
        if !self.variant.is_empty() {
            struct_ser.serialize_field("variant", &self.variant)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ResolveFloatResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "value",
            "reason",
            "variant",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Value,
            Reason,
            Variant,
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
                            "value" => Ok(GeneratedField::Value),
                            "reason" => Ok(GeneratedField::Reason),
                            "variant" => Ok(GeneratedField::Variant),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ResolveFloatResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct schema.v1.ResolveFloatResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<ResolveFloatResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut value__ = None;
                let mut reason__ = None;
                let mut variant__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Value => {
                            if value__.is_some() {
                                return Err(serde::de::Error::duplicate_field("value"));
                            }
                            value__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Reason => {
                            if reason__.is_some() {
                                return Err(serde::de::Error::duplicate_field("reason"));
                            }
                            reason__ = Some(map.next_value()?);
                        }
                        GeneratedField::Variant => {
                            if variant__.is_some() {
                                return Err(serde::de::Error::duplicate_field("variant"));
                            }
                            variant__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(ResolveFloatResponse {
                    value: value__.unwrap_or_default(),
                    reason: reason__.unwrap_or_default(),
                    variant: variant__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("schema.v1.ResolveFloatResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ResolveIntRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.flag_key.is_empty() {
            len += 1;
        }
        if self.context.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("schema.v1.ResolveIntRequest", len)?;
        if !self.flag_key.is_empty() {
            struct_ser.serialize_field("flagKey", &self.flag_key)?;
        }
        if let Some(v) = self.context.as_ref() {
            struct_ser.serialize_field("context", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ResolveIntRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "flag_key",
            "flagKey",
            "context",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            FlagKey,
            Context,
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
                            "flagKey" | "flag_key" => Ok(GeneratedField::FlagKey),
                            "context" => Ok(GeneratedField::Context),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ResolveIntRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct schema.v1.ResolveIntRequest")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<ResolveIntRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut flag_key__ = None;
                let mut context__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::FlagKey => {
                            if flag_key__.is_some() {
                                return Err(serde::de::Error::duplicate_field("flagKey"));
                            }
                            flag_key__ = Some(map.next_value()?);
                        }
                        GeneratedField::Context => {
                            if context__.is_some() {
                                return Err(serde::de::Error::duplicate_field("context"));
                            }
                            context__ = map.next_value()?;
                        }
                    }
                }
                Ok(ResolveIntRequest {
                    flag_key: flag_key__.unwrap_or_default(),
                    context: context__,
                })
            }
        }
        deserializer.deserialize_struct("schema.v1.ResolveIntRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ResolveIntResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.value != 0 {
            len += 1;
        }
        if !self.reason.is_empty() {
            len += 1;
        }
        if !self.variant.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("schema.v1.ResolveIntResponse", len)?;
        if self.value != 0 {
            struct_ser.serialize_field("value", ToString::to_string(&self.value).as_str())?;
        }
        if !self.reason.is_empty() {
            struct_ser.serialize_field("reason", &self.reason)?;
        }
        if !self.variant.is_empty() {
            struct_ser.serialize_field("variant", &self.variant)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ResolveIntResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "value",
            "reason",
            "variant",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Value,
            Reason,
            Variant,
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
                            "value" => Ok(GeneratedField::Value),
                            "reason" => Ok(GeneratedField::Reason),
                            "variant" => Ok(GeneratedField::Variant),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ResolveIntResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct schema.v1.ResolveIntResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<ResolveIntResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut value__ = None;
                let mut reason__ = None;
                let mut variant__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Value => {
                            if value__.is_some() {
                                return Err(serde::de::Error::duplicate_field("value"));
                            }
                            value__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Reason => {
                            if reason__.is_some() {
                                return Err(serde::de::Error::duplicate_field("reason"));
                            }
                            reason__ = Some(map.next_value()?);
                        }
                        GeneratedField::Variant => {
                            if variant__.is_some() {
                                return Err(serde::de::Error::duplicate_field("variant"));
                            }
                            variant__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(ResolveIntResponse {
                    value: value__.unwrap_or_default(),
                    reason: reason__.unwrap_or_default(),
                    variant: variant__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("schema.v1.ResolveIntResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ResolveObjectRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.flag_key.is_empty() {
            len += 1;
        }
        if self.context.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("schema.v1.ResolveObjectRequest", len)?;
        if !self.flag_key.is_empty() {
            struct_ser.serialize_field("flagKey", &self.flag_key)?;
        }
        if let Some(v) = self.context.as_ref() {
            struct_ser.serialize_field("context", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ResolveObjectRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "flag_key",
            "flagKey",
            "context",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            FlagKey,
            Context,
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
                            "flagKey" | "flag_key" => Ok(GeneratedField::FlagKey),
                            "context" => Ok(GeneratedField::Context),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ResolveObjectRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct schema.v1.ResolveObjectRequest")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<ResolveObjectRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut flag_key__ = None;
                let mut context__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::FlagKey => {
                            if flag_key__.is_some() {
                                return Err(serde::de::Error::duplicate_field("flagKey"));
                            }
                            flag_key__ = Some(map.next_value()?);
                        }
                        GeneratedField::Context => {
                            if context__.is_some() {
                                return Err(serde::de::Error::duplicate_field("context"));
                            }
                            context__ = map.next_value()?;
                        }
                    }
                }
                Ok(ResolveObjectRequest {
                    flag_key: flag_key__.unwrap_or_default(),
                    context: context__,
                })
            }
        }
        deserializer.deserialize_struct("schema.v1.ResolveObjectRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ResolveObjectResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.value.is_some() {
            len += 1;
        }
        if !self.reason.is_empty() {
            len += 1;
        }
        if !self.variant.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("schema.v1.ResolveObjectResponse", len)?;
        if let Some(v) = self.value.as_ref() {
            struct_ser.serialize_field("value", v)?;
        }
        if !self.reason.is_empty() {
            struct_ser.serialize_field("reason", &self.reason)?;
        }
        if !self.variant.is_empty() {
            struct_ser.serialize_field("variant", &self.variant)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ResolveObjectResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "value",
            "reason",
            "variant",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Value,
            Reason,
            Variant,
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
                            "value" => Ok(GeneratedField::Value),
                            "reason" => Ok(GeneratedField::Reason),
                            "variant" => Ok(GeneratedField::Variant),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ResolveObjectResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct schema.v1.ResolveObjectResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<ResolveObjectResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut value__ = None;
                let mut reason__ = None;
                let mut variant__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Value => {
                            if value__.is_some() {
                                return Err(serde::de::Error::duplicate_field("value"));
                            }
                            value__ = map.next_value()?;
                        }
                        GeneratedField::Reason => {
                            if reason__.is_some() {
                                return Err(serde::de::Error::duplicate_field("reason"));
                            }
                            reason__ = Some(map.next_value()?);
                        }
                        GeneratedField::Variant => {
                            if variant__.is_some() {
                                return Err(serde::de::Error::duplicate_field("variant"));
                            }
                            variant__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(ResolveObjectResponse {
                    value: value__,
                    reason: reason__.unwrap_or_default(),
                    variant: variant__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("schema.v1.ResolveObjectResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ResolveStringRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.flag_key.is_empty() {
            len += 1;
        }
        if self.context.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("schema.v1.ResolveStringRequest", len)?;
        if !self.flag_key.is_empty() {
            struct_ser.serialize_field("flagKey", &self.flag_key)?;
        }
        if let Some(v) = self.context.as_ref() {
            struct_ser.serialize_field("context", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ResolveStringRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "flag_key",
            "flagKey",
            "context",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            FlagKey,
            Context,
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
                            "flagKey" | "flag_key" => Ok(GeneratedField::FlagKey),
                            "context" => Ok(GeneratedField::Context),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ResolveStringRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct schema.v1.ResolveStringRequest")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<ResolveStringRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut flag_key__ = None;
                let mut context__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::FlagKey => {
                            if flag_key__.is_some() {
                                return Err(serde::de::Error::duplicate_field("flagKey"));
                            }
                            flag_key__ = Some(map.next_value()?);
                        }
                        GeneratedField::Context => {
                            if context__.is_some() {
                                return Err(serde::de::Error::duplicate_field("context"));
                            }
                            context__ = map.next_value()?;
                        }
                    }
                }
                Ok(ResolveStringRequest {
                    flag_key: flag_key__.unwrap_or_default(),
                    context: context__,
                })
            }
        }
        deserializer.deserialize_struct("schema.v1.ResolveStringRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ResolveStringResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.value.is_empty() {
            len += 1;
        }
        if !self.reason.is_empty() {
            len += 1;
        }
        if !self.variant.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("schema.v1.ResolveStringResponse", len)?;
        if !self.value.is_empty() {
            struct_ser.serialize_field("value", &self.value)?;
        }
        if !self.reason.is_empty() {
            struct_ser.serialize_field("reason", &self.reason)?;
        }
        if !self.variant.is_empty() {
            struct_ser.serialize_field("variant", &self.variant)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ResolveStringResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "value",
            "reason",
            "variant",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Value,
            Reason,
            Variant,
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
                            "value" => Ok(GeneratedField::Value),
                            "reason" => Ok(GeneratedField::Reason),
                            "variant" => Ok(GeneratedField::Variant),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ResolveStringResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct schema.v1.ResolveStringResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<ResolveStringResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut value__ = None;
                let mut reason__ = None;
                let mut variant__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Value => {
                            if value__.is_some() {
                                return Err(serde::de::Error::duplicate_field("value"));
                            }
                            value__ = Some(map.next_value()?);
                        }
                        GeneratedField::Reason => {
                            if reason__.is_some() {
                                return Err(serde::de::Error::duplicate_field("reason"));
                            }
                            reason__ = Some(map.next_value()?);
                        }
                        GeneratedField::Variant => {
                            if variant__.is_some() {
                                return Err(serde::de::Error::duplicate_field("variant"));
                            }
                            variant__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(ResolveStringResponse {
                    value: value__.unwrap_or_default(),
                    reason: reason__.unwrap_or_default(),
                    variant: variant__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("schema.v1.ResolveStringResponse", FIELDS, GeneratedVisitor)
    }
}
