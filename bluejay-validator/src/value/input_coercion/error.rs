use crate::Path;
use bluejay_core::{ObjectValue, Value};
#[cfg(feature = "parser-integration")]
use bluejay_parser::{
    ast::Value as ParserValue,
    error::{Annotation, Error as ParserError},
    HasSpan,
};
#[cfg(feature = "parser-integration")]
use itertools::Itertools;
use std::borrow::Cow;

#[derive(PartialEq, Debug)]
pub enum Error<'a, const CONST: bool, V: Value<CONST>> {
    NullValueForRequiredType {
        value: &'a V,
        input_type_name: String,
        path: Path<'a>,
    },
    NoImplicitConversion {
        value: &'a V,
        input_type_name: String,
        path: Path<'a>,
    },
    NoEnumMemberWithName {
        name: &'a str,
        value: &'a V,
        enum_type_name: &'a str,
        path: Path<'a>,
    },
    NoValueForRequiredFields {
        value: &'a V,
        field_names: Vec<&'a str>,
        input_object_type_name: &'a str,
        path: Path<'a>,
    },
    NonUniqueFieldNames {
        value: &'a V,
        field_name: &'a str,
        keys: Vec<&'a <V::Object as ObjectValue<CONST>>::Key>,
        path: Path<'a>,
    },
    NoInputFieldWithName {
        field: &'a <V::Object as ObjectValue<CONST>>::Key,
        input_object_type_name: &'a str,
        path: Path<'a>,
    },
    CustomScalarInvalidValue {
        value: &'a V,
        custom_scalar_type_name: &'a str,
        message: Cow<'static, str>,
        path: Path<'a>,
    },
    #[cfg(feature = "one-of-input-objects")]
    OneOfInputNullValues {
        value: &'a V,
        input_object_type_name: &'a str,
        null_entries: Vec<(&'a <V::Object as ObjectValue<CONST>>::Key, &'a V)>,
        path: Path<'a>,
    },
    #[cfg(feature = "one-of-input-objects")]
    OneOfInputNotSingleNonNullValue {
        value: &'a V,
        input_object_type_name: &'a str,
        non_null_entries: Vec<(&'a <V::Object as ObjectValue<CONST>>::Key, &'a V)>,
        path: Path<'a>,
    },
}

impl<const CONST: bool, V: Value<CONST>> Error<'_, CONST, V> {
    pub fn message(&self) -> Cow<'static, str> {
        match self {
            Self::NullValueForRequiredType { input_type_name, .. } => {
                format!("Got null when non-null value of type {input_type_name} was expected")
                    .into()
            }
            Self::NoImplicitConversion { input_type_name, value, .. } => {
                format!("No implicit conversion of {} to {input_type_name}", value.as_ref().variant()).into()
            }
            Self::NoEnumMemberWithName { name, enum_type_name, .. } => {
                format!("No member `{name}` on enum {enum_type_name}").into()
            }
            Self::NoValueForRequiredFields {
                field_names, input_object_type_name, ..
            } => {
                let joined_field_names = field_names.iter().join(", ");
                format!(
                    "No value for required fields on input type {input_object_type_name}: {joined_field_names}"
                )
                .into()
            }
            Self::NonUniqueFieldNames { field_name, .. } => {
                format!("Object with multiple entries for field {field_name}").into()
            }
            Self::NoInputFieldWithName { field, input_object_type_name, .. } => {
                format!(
                    "No field with name {} on input type {input_object_type_name}",
                    field.as_ref()
                )
                .into()
            }
            Self::CustomScalarInvalidValue { message, .. } => message.clone(),
            #[cfg(feature = "one-of-input-objects")]
            Self::OneOfInputNullValues { input_object_type_name, .. } => {
                format!("Multiple entries with null values for oneOf input object {input_object_type_name}")
                    .into()
            }
            #[cfg(feature = "one-of-input-objects")]
            Self::OneOfInputNotSingleNonNullValue { input_object_type_name, non_null_entries, .. } => {
                format!(
                    "Got {} entries with non-null values for oneOf input object {input_object_type_name}",
                    non_null_entries.len()
                )
                .into()
            }
        }
    }
}

#[cfg(feature = "parser-integration")]
impl<'a, const CONST: bool> From<Error<'a, CONST, ParserValue<'a, CONST>>> for ParserError {
    fn from(error: Error<'a, CONST, ParserValue<'a, CONST>>) -> Self {
        match &error {
            Error::NullValueForRequiredType { value, .. } => Self::new(
                error.message(),
                Some(Annotation::new(
                    "Expected non-null value",
                    value.span().clone(),
                )),
                Vec::new(),
            ),
            Error::NoImplicitConversion {
                value,
                input_type_name,
                ..
            } => Self::new(
                error.message(),
                Some(Annotation::new(
                    format!("No implicit conversion to {input_type_name}"),
                    value.span().clone(),
                )),
                Vec::new(),
            ),
            Error::NoEnumMemberWithName {
                value,
                enum_type_name,
                ..
            } => Self::new(
                error.message(),
                Some(Annotation::new(
                    format!("No such member on enum {enum_type_name}"),
                    value.span().clone(),
                )),
                Vec::new(),
            ),
            Error::NoValueForRequiredFields {
                value, field_names, ..
            } => {
                let joined_field_names = field_names.iter().join(", ");
                Self::new(
                    error.message(),
                    Some(Annotation::new(
                        format!("No value for required fields: {joined_field_names}"),
                        value.span().clone(),
                    )),
                    Vec::new(),
                )
            }
            Error::NonUniqueFieldNames { keys, .. } => Self::new(
                error.message(),
                None,
                Vec::from_iter(
                    keys.iter()
                        .map(|key| Annotation::new("Entry for field", key.span().clone())),
                ),
            ),
            Error::NoInputFieldWithName {
                field,
                input_object_type_name,
                ..
            } => Self::new(
                error.message(),
                Some(Annotation::new(
                    format!("No field with this name on input type {input_object_type_name}"),
                    field.span().clone(),
                )),
                Vec::new(),
            ),
            Error::CustomScalarInvalidValue { value, message, .. } => Self::new(
                message.clone(),
                Some(Annotation::new(message.clone(), value.span().clone())),
                Vec::new(),
            ),
            #[cfg(feature = "one-of-input-objects")]
            Error::OneOfInputNullValues {
                value,
                null_entries,
                ..
            } => Self::new(
                error.message(),
                Some(Annotation::new(
                    "oneOf input object must not contain any null values",
                    value.span().clone(),
                )),
                null_entries
                    .iter()
                    .map(|(key, value)| {
                        Annotation::new("Entry with null value", key.span().merge(value.span()))
                    })
                    .collect(),
            ),
            #[cfg(feature = "one-of-input-objects")]
            Error::OneOfInputNotSingleNonNullValue {
                value,
                non_null_entries,
                ..
            } => Self::new(
                error.message(),
                Some(Annotation::new(
                    "oneOf input object must contain single non-null",
                    value.span().clone(),
                )),
                non_null_entries
                    .iter()
                    .map(|(key, value)| {
                        Annotation::new("Entry with non-null value", key.span().merge(value.span()))
                    })
                    .collect(),
            ),
        }
    }
}
