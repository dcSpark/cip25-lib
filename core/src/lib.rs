use std::io::{BufRead, Seek, Write};
use prelude::*;

// This library was code-generated using an experimental CDDL to rust tool:
// https://github.com/Emurgo/cddl-codegen

use cbor_event::{self, de::Deserializer, se::Serializer};

use cbor_event::Type as CBORType;

use cbor_event::Special as CBORSpecial;

use serialization::*;

pub mod prelude;

pub mod serialization;

use std::collections::BTreeMap;

use std::convert::{From, TryFrom};

use linked_hash_map::LinkedHashMap;

type AssetName = Vec<u8>;

type Data = LinkedHashMap<PolicyId, LinkedHashMap<AssetName, MetadataDetails>>;

type PolicyId = Vec<u8>;

type PolicyIds = Vec<PolicyId>;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FilesDetails {
    pub name: String64,
    name_key_encoding: StringEncoding,
    pub media_type: String64,
    media_type_key_encoding: StringEncoding,
    pub src: String64OrArrString64,
    src_key_encoding: StringEncoding,
    encoding: LenEncoding,
    orig_deser_order: Option<Vec<usize>>,
}

impl FilesDetails {
    pub fn new(name: String64, media_type: String64, src: String64OrArrString64) -> Self {
        Self {
            name,
            name_key_encoding: StringEncoding::default(),
            media_type,
            media_type_key_encoding: StringEncoding::default(),
            src,
            src_key_encoding: StringEncoding::default(),
            encoding: LenEncoding::default(),
            orig_deser_order: None,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LabelMetadata {
    pub data: Data,
    data_encoding: LenEncoding,
    data_key_encodings: BTreeMap<Vec<u8>, StringEncoding>,
    data_value_encodings: BTreeMap<Vec<u8>, (LenEncoding, BTreeMap<Vec<u8>, StringEncoding>)>,
    data_key_encoding: StringEncoding,
    version_encoding: Option<cbor_event::Sz>,
    version_key_encoding: StringEncoding,
    encoding: LenEncoding,
    orig_deser_order: Option<Vec<usize>>,
}

impl LabelMetadata {
    pub fn new(data: Data) -> Self {
        Self {
            data,
            data_encoding: LenEncoding::default(),
            data_key_encodings: BTreeMap::new(),
            data_value_encodings: BTreeMap::new(),
            data_key_encoding: StringEncoding::default(),
            version_encoding: None,
            version_key_encoding: StringEncoding::default(),
            encoding: LenEncoding::default(),
            orig_deser_order: None,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Metadata {
    pub key_721: LabelMetadata,
    key_721_key_encoding: Option<cbor_event::Sz>,
    encoding: LenEncoding,
    orig_deser_order: Option<Vec<usize>>,
}

impl Metadata {
    pub fn new(key_721: LabelMetadata) -> Self {
        Self {
            key_721,
            key_721_key_encoding: None,
            encoding: LenEncoding::default(),
            orig_deser_order: None,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MetadataDetails {
    pub name: String64,
    name_key_encoding: StringEncoding,
    pub image: String64OrArrString64,
    image_key_encoding: StringEncoding,
    pub media_type: Option<String64>,
    media_type_key_encoding: StringEncoding,
    pub description: Option<String64OrArrString64>,
    description_key_encoding: StringEncoding,
    pub files: Option<Vec<FilesDetails>>,
    files_encoding: LenEncoding,
    files_key_encoding: StringEncoding,
    encoding: LenEncoding,
    orig_deser_order: Option<Vec<usize>>,
}

impl MetadataDetails {
    pub fn new(name: String64, image: String64OrArrString64) -> Self {
        Self {
            name,
            name_key_encoding: StringEncoding::default(),
            image,
            image_key_encoding: StringEncoding::default(),
            media_type: None,
            media_type_key_encoding: StringEncoding::default(),
            description: None,
            description_key_encoding: StringEncoding::default(),
            files: None,
            files_encoding: LenEncoding::default(),
            files_key_encoding: StringEncoding::default(),
            encoding: LenEncoding::default(),
            orig_deser_order: None,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct String64 {
    inner: String,
    inner_encoding: StringEncoding,
}

impl String64 {
    pub fn get(&self) -> &String {
        &self.inner
    }

    pub fn new(inner: String) -> Result<Self, DeserializeError> {
        if inner.len() > 64 {
            return Err(DeserializeError::new("String64", DeserializeFailure::RangeCheck{ found: inner.len(), min: Some(0), max: Some(64) }));
        }
        Ok(Self {
            inner,
            inner_encoding: StringEncoding::default(),
        })
    }
}

impl TryFrom<String> for String64 {
    type Error = DeserializeError;

    fn try_from(inner: String) -> Result<Self, Self::Error> {
        String64::new(inner)
    }
}

impl From<String64> for String {
    fn from(wrapper: String64) -> Self {
        wrapper.inner
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum String64OrArrString64 {
    String64(String64),
    ArrString64(Vec<String64>, LenEncoding),
}