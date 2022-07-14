use wasm_bindgen::prelude::*;

mod prelude;

use std::collections::BTreeMap;

use linked_hash_map::LinkedHashMap;

use core::serialization::{LenEncoding, StringEncoding};

type AssetName = Vec<u8>;

type PolicyId = Vec<u8>;

#[wasm_bindgen]

#[derive(Clone, Debug)]
pub struct MapAssetNameToU64(LinkedHashMap<AssetName, u64>);

#[wasm_bindgen]

impl MapAssetNameToU64 {
    pub fn new() -> Self {
        Self(LinkedHashMap::new())
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn insert(&mut self, key: AssetName, value: u64) -> Option<u64> {
        self.0.insert(key, value)
    }

    pub fn get(&self, key: AssetName) -> Option<u64> {
        self.0.get(&key).copied()
    }

    pub fn keys(&self) -> AssetNames {
        AssetNames(self.0.iter().map(|(k, _v)| k.clone()).collect::<Vec<_>>())
    }
}

impl From<LinkedHashMap<AssetName, u64>> for MapAssetNameToU64 {
    fn from(native: LinkedHashMap<AssetName, u64>) -> Self {
        Self(native)
    }
}

impl std::convert::Into<LinkedHashMap<AssetName, u64>> for MapAssetNameToU64 {
    fn into(self) -> LinkedHashMap<AssetName, u64> {
        self.0
    }
}

#[wasm_bindgen]

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AssetNames(Vec<AssetName>);

#[wasm_bindgen]

impl AssetNames {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn get(&self, index: usize) -> AssetName {
        self.0[index].clone()
    }

    pub fn add(&mut self, elem: AssetName) {
        self.0.push(elem);
    }
}

impl From<Vec<AssetName>> for AssetNames {
    fn from(native: Vec<AssetName>) -> Self {
        Self(native)
    }
}

impl std::convert::Into<Vec<AssetName>> for AssetNames {
    fn into(self) -> Vec<AssetName> {
        self.0
    }
}

#[wasm_bindgen]

#[derive(Clone, Debug)]
pub struct Data(LinkedHashMap<PolicyId, LinkedHashMap<AssetName, u64>>);

#[wasm_bindgen]

impl Data {
    pub fn new() -> Self {
        Self(LinkedHashMap::new())
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn insert(&mut self, key: PolicyId, value: &MapAssetNameToU64) -> Option<MapAssetNameToU64> {
        self.0.insert(key, value.clone().into()).map(|v| v.clone().into())
    }

    pub fn get(&self, key: PolicyId) -> Option<MapAssetNameToU64> {
        self.0.get(&key).map(|v| v.clone().into())
    }

    pub fn keys(&self) -> PolicyIds {
        PolicyIds(self.0.iter().map(|(k, _v)| k.clone()).collect::<Vec<_>>())
    }
}

impl From<LinkedHashMap<PolicyId, LinkedHashMap<AssetName, u64>>> for Data {
    fn from(native: LinkedHashMap<PolicyId, LinkedHashMap<AssetName, u64>>) -> Self {
        Self(native)
    }
}

impl std::convert::Into<LinkedHashMap<PolicyId, LinkedHashMap<AssetName, u64>>> for Data {
    fn into(self) -> LinkedHashMap<PolicyId, LinkedHashMap<AssetName, u64>> {
        self.0
    }
}

#[wasm_bindgen]

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct String64s(Vec<core::String64>);

#[wasm_bindgen]

impl String64s {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn get(&self, index: usize) -> String64 {
        self.0[index].clone().into()
    }

    pub fn add(&mut self, elem: &String64) {
        self.0.push(elem.clone().into());
    }
}

impl From<Vec<core::String64>> for String64s {
    fn from(native: Vec<core::String64>) -> Self {
        Self(native)
    }
}

impl std::convert::Into<Vec<core::String64>> for String64s {
    fn into(self) -> Vec<core::String64> {
        self.0
    }
}

#[wasm_bindgen]

#[derive(Clone, Debug)]
pub struct FilesDetails(core::FilesDetails);

#[wasm_bindgen]

impl FilesDetails {
    pub fn name(&self) -> String64 {
        self.0.name.clone().into()
    }

    pub fn media_type(&self) -> String64 {
        self.0.media_type.clone().into()
    }

    pub fn src(&self) -> String64OrArrString64 {
        self.0.src.clone().into()
    }

    pub fn new(name: &String64, media_type: &String64, src: &String64OrArrString64) -> Self {
        Self(core::FilesDetails::new(name.clone().into(), media_type.clone().into(), src.clone().into()))
    }
}

impl From<core::FilesDetails> for FilesDetails {
    fn from(native: core::FilesDetails) -> Self {
        Self(native)
    }
}

impl From<FilesDetails> for core::FilesDetails {
    fn from(wasm: FilesDetails) -> Self {
        wasm.0
    }
}

#[wasm_bindgen]

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PolicyIds(Vec<PolicyId>);

#[wasm_bindgen]

impl PolicyIds {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn get(&self, index: usize) -> PolicyId {
        self.0[index].clone()
    }

    pub fn add(&mut self, elem: PolicyId) {
        self.0.push(elem);
    }
}

impl From<Vec<PolicyId>> for PolicyIds {
    fn from(native: Vec<PolicyId>) -> Self {
        Self(native)
    }
}

impl std::convert::Into<Vec<PolicyId>> for PolicyIds {
    fn into(self) -> Vec<PolicyId> {
        self.0
    }
}

#[wasm_bindgen]

#[derive(Clone, Debug)]
pub struct LabelMetadata(core::LabelMetadata);

#[wasm_bindgen]

impl LabelMetadata {
    pub fn data(&self) -> Data {
        self.0.data.clone().into()
    }

    pub fn new(data: Data) -> Self {
        Self(core::LabelMetadata::new(data.clone().into()))
    }
}

impl From<core::LabelMetadata> for LabelMetadata {
    fn from(native: core::LabelMetadata) -> Self {
        Self(native)
    }
}

impl From<LabelMetadata> for core::LabelMetadata {
    fn from(wasm: LabelMetadata) -> Self {
        wasm.0
    }
}

#[wasm_bindgen]

#[derive(Clone, Debug)]
pub struct Metadata(core::Metadata);

#[wasm_bindgen]

impl Metadata {
    pub fn key_721(&self) -> LabelMetadata {
        self.0.key_721.clone().into()
    }

    pub fn new(key_721: &LabelMetadata) -> Self {
        Self(core::Metadata::new(key_721.clone().into()))
    }
}

impl From<core::Metadata> for Metadata {
    fn from(native: core::Metadata) -> Self {
        Self(native)
    }
}

impl From<Metadata> for core::Metadata {
    fn from(wasm: Metadata) -> Self {
        wasm.0
    }
}

#[wasm_bindgen]

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FilesDetailss(Vec<core::FilesDetails>);

#[wasm_bindgen]

impl FilesDetailss {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn get(&self, index: usize) -> FilesDetails {
        self.0[index].clone().into()
    }

    pub fn add(&mut self, elem: &FilesDetails) {
        self.0.push(elem.clone().into());
    }
}

impl From<Vec<core::FilesDetails>> for FilesDetailss {
    fn from(native: Vec<core::FilesDetails>) -> Self {
        Self(native)
    }
}

impl std::convert::Into<Vec<core::FilesDetails>> for FilesDetailss {
    fn into(self) -> Vec<core::FilesDetails> {
        self.0
    }
}

#[wasm_bindgen]

#[derive(Clone, Debug)]
pub struct MetadataDetails(core::MetadataDetails);

#[wasm_bindgen]

impl MetadataDetails {
    pub fn name(&self) -> String64 {
        self.0.name.clone().into()
    }

    pub fn image(&self) -> String64OrArrString64 {
        self.0.image.clone().into()
    }

    pub fn set_media_type(&mut self, media_type: &String64) {
        self.0.media_type = Some(media_type.clone().into())
    }

    pub fn media_type(&self) -> Option<String64> {
        self.0.media_type.clone().map(std::convert::Into::into)
    }

    pub fn set_description(&mut self, description: &String64OrArrString64) {
        self.0.description = Some(description.clone().into())
    }

    pub fn description(&self) -> Option<String64OrArrString64> {
        self.0.description.clone().map(std::convert::Into::into)
    }

    pub fn set_files(&mut self, files: &FilesDetailss) {
        self.0.files = Some(files.clone().into())
    }

    pub fn files(&self) -> Option<FilesDetailss> {
        self.0.files.clone().map(std::convert::Into::into)
    }

    pub fn new(name: &String64, image: &String64OrArrString64) -> Self {
        Self(core::MetadataDetails::new(name.clone().into(), image.clone().into()))
    }
}

impl From<core::MetadataDetails> for MetadataDetails {
    fn from(native: core::MetadataDetails) -> Self {
        Self(native)
    }
}

impl From<MetadataDetails> for core::MetadataDetails {
    fn from(wasm: MetadataDetails) -> Self {
        wasm.0
    }
}

#[wasm_bindgen]

#[derive(Clone, Debug)]
pub struct String64(core::String64);

#[wasm_bindgen]

impl String64 {
    pub fn get(&self) -> String {
        self.0.get().clone().clone()
    }
}

impl From<core::String64> for String64 {
    fn from(native: core::String64) -> Self {
        Self(native)
    }
}

impl From<String64> for core::String64 {
    fn from(wasm: String64) -> Self {
        wasm.0
    }
}

#[wasm_bindgen]

pub enum String64OrArrString64Kind {
    String64,
    ArrString64,
}

#[wasm_bindgen]

#[derive(Clone, Debug)]
pub struct String64OrArrString64(core::String64OrArrString64);

#[wasm_bindgen]

impl String64OrArrString64 {
    pub fn new_string64(string64: &String64) -> Self {
        Self(core::String64OrArrString64::String64(string64.clone().into()))
    }

    pub fn new_arr_string64(arr_string64: &String64s) -> Self {
        Self(core::String64OrArrString64::ArrString64(arr_string64.clone().into(), LenEncoding::default()))
    }

    pub fn kind(&self) -> String64OrArrString64Kind {
        match &self.0 {
            core::String64OrArrString64::String64(_x) => String64OrArrString64Kind::String64,
            core::String64OrArrString64::ArrString64(_x, _x_encoding) => String64OrArrString64Kind::ArrString64,
        }
    }

    pub fn as_string64(&self) -> Option<String64> {
        match &self.0 {
            core::String64OrArrString64::String64(variant) => Some(variant.clone().into()),
            _ => None,
        }
    }

    pub fn as_arr_string64(&self) -> Option<String64s> {
        match &self.0 {
            core::String64OrArrString64::ArrString64(variant, variant_encoding) => Some(variant.clone().into()),
            _ => None,
        }
    }
}

impl From<core::String64OrArrString64> for String64OrArrString64 {
    fn from(native: core::String64OrArrString64) -> Self {
        Self(native)
    }
}

impl From<String64OrArrString64> for core::String64OrArrString64 {
    fn from(wasm: String64OrArrString64) -> Self {
        wasm.0
    }
}