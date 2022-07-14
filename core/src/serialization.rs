// what to put here?
use cbor_event::{Sz, LenSz, StringLenSz};

pub struct CBORReadLen {
    deser_len: cbor_event::LenSz,
    read: u64,
}

impl CBORReadLen {
    pub fn new(len: cbor_event::LenSz) -> Self {
        Self {
            deser_len: len,
            read: 0,
        }
    }

    // Marks {n} values as being read, and if we go past the available definite length
    // given by the CBOR, we return an error.
    pub fn read_elems(&mut self, count: usize) -> Result<(), DeserializeFailure> {
        match self.deser_len {
            cbor_event::LenSz::Len(n, _) => {
                self.read += count as u64;
                if self.read > n {
                    Err(DeserializeFailure::DefiniteLenMismatch(n, None))
                } else {
                    Ok(())
                }
            },
            cbor_event::LenSz::Indefinite => Ok(()),
        }
    }

    pub fn finish(&self) -> Result<(), DeserializeFailure> {
        match self.deser_len {
            cbor_event::LenSz::Len(n, _) => {
                if self.read == n {
                    Ok(())
                } else {
                    Err(DeserializeFailure::DefiniteLenMismatch(n, Some(self.read)))
                }
            },
            cbor_event::LenSz::Indefinite => Ok(()),
        }
    }
}

#[inline]
fn sz_max(sz: cbor_event::Sz) -> u64 {
    match sz {
        Sz::Inline => 23u64,
        Sz::One => u8::MAX as u64,
        Sz::Two => u16::MAX as u64,
        Sz::Four => u32::MAX as u64,
        Sz::Eight => u64::MAX,
    }
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum LenEncoding {
    Canonical,
    Definite(cbor_event::Sz),
    Indefinite,
}

impl Default for LenEncoding {
    fn default() -> Self {
        Self::Canonical
    }
}

impl From<cbor_event::LenSz> for LenEncoding {
    fn from(len_sz: cbor_event::LenSz) -> Self {
        match len_sz {
            cbor_event::LenSz::Len(len, sz) => if cbor_event::Sz::canonical(len) == sz {
                Self::Canonical
            } else {
                Self::Definite(sz)
            },
            cbor_event::LenSz::Indefinite => Self::Indefinite,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum StringEncoding {
    Canonical,
    Indefinite(Vec<(u64, Sz)>),
    Definite(Sz),
}

impl Default for StringEncoding {
    fn default() -> Self {
        Self::Canonical
    }
}

impl From<cbor_event::StringLenSz> for StringEncoding {
    fn from(len_sz: cbor_event::StringLenSz) -> Self {
        match len_sz {
            cbor_event::StringLenSz::Len(sz) => Self::Definite(sz),
            cbor_event::StringLenSz::Indefinite(lens) => Self::Indefinite(lens),
        }
    }
}#[inline]
fn fit_sz(len: u64, sz: Option<cbor_event::Sz>, force_canonical: bool) -> Sz {
    match sz {
        Some(sz) => if !force_canonical && len <= sz_max(sz) {
            sz
        } else {
            Sz::canonical(len)
        },
        None => Sz::canonical(len),
    }
}

impl LenEncoding {
    pub fn to_len_sz(&self, len: u64, force_canonical: bool) -> cbor_event::LenSz {
        if force_canonical {
            cbor_event::LenSz::Len(len, cbor_event::Sz::canonical(len))
        } else {
            match self {
                Self::Canonical => cbor_event::LenSz::Len(len, cbor_event::Sz::canonical(len)),
                Self::Definite(sz) => if sz_max(*sz) >= len {
                    cbor_event::LenSz::Len(len, *sz)
                } else {
                    cbor_event::LenSz::Len(len, cbor_event::Sz::canonical(len))
                },
                Self::Indefinite => cbor_event::LenSz::Indefinite,
            }
        }
    }

    pub fn end<'a, W: Write + Sized>(&self, serializer: &'a mut Serializer<W>, force_canonical: bool) -> cbor_event::Result<&'a mut Serializer<W>> {
        if !force_canonical && *self == Self::Indefinite {
            serializer.write_special(CBORSpecial::Break)?;
        }
        Ok(serializer)
    }
}

impl StringEncoding {
    pub fn to_str_len_sz(&self, len: u64, force_canonical: bool) -> cbor_event::StringLenSz {
        if force_canonical {
            cbor_event::StringLenSz::Len(cbor_event::Sz::canonical(len))
        } else {
            match self {
                Self::Canonical => cbor_event::StringLenSz::Len(cbor_event::Sz::canonical(len)),
                Self::Definite(sz) => if sz_max(*sz) >= len {
                    cbor_event::StringLenSz::Len(*sz)
                } else {
                    cbor_event::StringLenSz::Len(cbor_event::Sz::canonical(len))
                },
                Self::Indefinite(lens) => cbor_event::StringLenSz::Indefinite(lens.clone()),
            }
        }
    }
}

pub trait Serialize {
    fn serialize<'a, W: Write + Sized>(
        &self,
        serializer: &'a mut Serializer<W>,
        force_canonical: bool,
    ) -> cbor_event::Result<&'a mut Serializer<W>>;
}

impl<T: cbor_event::se::Serialize> Serialize for T {
    fn serialize<'a, W: Write + Sized>(
        &self,
        serializer: &'a mut Serializer<W>,
        _force_canonical: bool,
    ) -> cbor_event::Result<&'a mut Serializer<W>> {
        <T as cbor_event::se::Serialize>::serialize(self, serializer)
    }
}

pub trait SerializeEmbeddedGroup {
    fn serialize_as_embedded_group<'a, W: Write + Sized>(
        &self,
        serializer: &'a mut Serializer<W>,
        force_canonical: bool,
    ) -> cbor_event::Result<&'a mut Serializer<W>>;
}


pub trait ToBytes {
    fn to_bytes(&self, force_canonical: bool) -> Vec<u8>;
}

impl<T: Serialize> ToBytes for T {
    fn to_bytes(&self, force_canonical: bool) -> Vec<u8> {
        let mut buf = Serializer::new_vec();
        self.serialize(&mut buf, force_canonical).unwrap();
        buf.finalize()
    }
}
use super::*;
use std::io::{Seek, SeekFrom};

impl Serialize for FilesDetails {
    fn serialize<'se, W: Write>(&self, serializer: &'se mut Serializer<W>, force_canonical: bool) -> cbor_event::Result<&'se mut Serializer<W>> {
        serializer.write_map_sz(self.encoding.to_len_sz(3, force_canonical))?;
        let deser_order = if !force_canonical && self.orig_deser_order.as_ref().map(|v| v.len() == 3).unwrap_or(false) {
            self.orig_deser_order.clone().unwrap()
        }
        else {
            vec![2,0,1]
        };
        for field_index in deser_order {
            match field_index {
                2 => {
                    serializer.write_text_sz(&"src", self.src_key_encoding.to_str_len_sz("src".len() as u64, force_canonical))?;
                    self.src.serialize(serializer, force_canonical)?;
                }
                0 => {
                    serializer.write_text_sz(&"name", self.name_key_encoding.to_str_len_sz("name".len() as u64, force_canonical))?;
                    self.name.serialize(serializer, force_canonical)?;
                }
                1 => {
                    serializer.write_text_sz(&"mediaType", self.media_type_key_encoding.to_str_len_sz("mediaType".len() as u64, force_canonical))?;
                    self.media_type.serialize(serializer, force_canonical)?;
                }
                _ => unreachable!()
            };
        }
        self.encoding.end(serializer, force_canonical)
    }
}

impl Deserialize for FilesDetails {
    fn deserialize<R: BufRead + Seek>(raw: &mut Deserializer<R>) -> Result<Self, DeserializeError> {
        (|| -> Result<_, DeserializeError> {
            let len = raw.map_sz()?;
            let encoding = len.into();
            let mut read_len = CBORReadLen::new(len);
            read_len.read_elems(3)?;
            let mut orig_deser_order = Vec::new();
            let mut src_key_encoding = StringEncoding::default();
            let mut src = None;
            let mut name_key_encoding = StringEncoding::default();
            let mut name = None;
            let mut media_type_key_encoding = StringEncoding::default();
            let mut media_type = None;
            let mut read = 0;
            while match len { cbor_event::LenSz::Len(n, _) => read < n as usize, cbor_event::LenSz::Indefinite => true, } {
                match raw.cbor_type()? {
                    CBORType::UnsignedInteger => match raw.unsigned_integer_sz()? {
                        (unknown_key, _enc) => return Err(DeserializeFailure::UnknownKey(Key::Uint(unknown_key)).into()),
                    },
                    CBORType::Text => {
                        let (text_key, key_enc) = raw.text_sz()?;
                        match text_key.as_str() {
                            "src" =>  {
                                if src.is_some() {
                                    return Err(DeserializeFailure::DuplicateKey(Key::Str("src".into())).into());
                                }
                                let tmp_src = (|| -> Result<_, DeserializeError> {
                                    Ok(String64OrArrString64::deserialize(raw)?)
                                })().map_err(|e| e.annotate("src"))?;
                                src = Some(tmp_src);
                                src_key_encoding = StringEncoding::from(key_enc);
                                orig_deser_order.push(2);
                            },
                            "name" =>  {
                                if name.is_some() {
                                    return Err(DeserializeFailure::DuplicateKey(Key::Str("name".into())).into());
                                }
                                let tmp_name = (|| -> Result<_, DeserializeError> {
                                    Ok(String64::deserialize(raw)?)
                                })().map_err(|e| e.annotate("name"))?;
                                name = Some(tmp_name);
                                name_key_encoding = StringEncoding::from(key_enc);
                                orig_deser_order.push(0);
                            },
                            "mediaType" =>  {
                                if media_type.is_some() {
                                    return Err(DeserializeFailure::DuplicateKey(Key::Str("mediaType".into())).into());
                                }
                                let tmp_media_type = (|| -> Result<_, DeserializeError> {
                                    Ok(String64::deserialize(raw)?)
                                })().map_err(|e| e.annotate("media_type"))?;
                                media_type = Some(tmp_media_type);
                                media_type_key_encoding = StringEncoding::from(key_enc);
                                orig_deser_order.push(1);
                            },
                            unknown_key => return Err(DeserializeFailure::UnknownKey(Key::Str(unknown_key.to_owned())).into()),
                        }
                    },
                    CBORType::Special => match len {
                        cbor_event::LenSz::Len(_, _) => return Err(DeserializeFailure::BreakInDefiniteLen.into()),
                        cbor_event::LenSz::Indefinite => match raw.special()? {
                            CBORSpecial::Break => break,
                            _ => return Err(DeserializeFailure::EndingBreakMissing.into()),
                        },
                    },
                    other_type => return Err(DeserializeFailure::UnexpectedKeyType(other_type).into()),
                }
                read += 1;
            }
            let name = match name {
                Some(x) => x,
                None => return Err(DeserializeFailure::MandatoryFieldMissing(Key::Str(String::from("name"))).into()),
            };
            let media_type = match media_type {
                Some(x) => x,
                None => return Err(DeserializeFailure::MandatoryFieldMissing(Key::Str(String::from("mediaType"))).into()),
            };
            let src = match src {
                Some(x) => x,
                None => return Err(DeserializeFailure::MandatoryFieldMissing(Key::Str(String::from("src"))).into()),
            };
            ();
            Ok(Self {
                name,
                name_key_encoding,
                media_type,
                media_type_key_encoding,
                src,
                src_key_encoding,
                encoding,
                orig_deser_order: Some(orig_deser_order),
            })
        })().map_err(|e| e.annotate("FilesDetails"))
    }
}

impl Serialize for LabelMetadata {
    fn serialize<'se, W: Write>(&self, serializer: &'se mut Serializer<W>, force_canonical: bool) -> cbor_event::Result<&'se mut Serializer<W>> {
        serializer.write_map_sz(self.encoding.to_len_sz(2, force_canonical))?;
        let deser_order = if !force_canonical && self.orig_deser_order.as_ref().map(|v| v.len() == 2).unwrap_or(false) {
            self.orig_deser_order.clone().unwrap()
        }
        else {
            vec![0,1]
        };
        for field_index in deser_order {
            match field_index {
                0 => {
                    serializer.write_text_sz(&"data", self.data_key_encoding.to_str_len_sz("data".len() as u64, force_canonical))?;
                    serializer.write_map_sz(self.data_encoding.to_len_sz(self.data.len() as u64, force_canonical))?;
                    let mut key_order = self.data.iter().map(|(k, v)| {
                        let mut buf = cbor_event::se::Serializer::new_vec();
                        let data_key_encoding = self.data_key_encodings.get(k).map(|e| e.clone()).unwrap_or_else(|| StringEncoding::default());
                        buf.write_bytes_sz(&k, data_key_encoding.to_str_len_sz(k.len() as u64, force_canonical))?;
                        Ok((buf.finalize(), k, v))
                    }).collect::<Result<Vec<(Vec<u8>, &_, &_)>, cbor_event::Error>>()?;
                    if force_canonical {
                        key_order.sort_by(|(lhs_bytes, _, _), (rhs_bytes, _, _)| {
                            match lhs_bytes.len().cmp(&rhs_bytes.len()) {
                                std::cmp::Ordering::Equal => lhs_bytes.cmp(&rhs_bytes),
                                diff_ord => diff_ord,
                            }
                        });
                    }
                    for (key_bytes, key, value) in key_order {
                        serializer.write_raw_bytes(&key_bytes)?;
                        let (data_value_encoding, data_value_key_encodings) = self.data_value_encodings.get(key).map(|e| e.clone()).unwrap_or_else(|| (LenEncoding::default(), BTreeMap::new()));
                        serializer.write_map_sz(data_value_encoding.to_len_sz(value.len() as u64, force_canonical))?;
                        let mut key_order = value.iter().map(|(k, v)| {
                            let mut buf = cbor_event::se::Serializer::new_vec();
                            let data_value_key_encoding = data_value_key_encodings.get(k).map(|e| e.clone()).unwrap_or_else(|| StringEncoding::default());
                            buf.write_bytes_sz(&k, data_value_key_encoding.to_str_len_sz(k.len() as u64, force_canonical))?;
                            Ok((buf.finalize(), k, v))
                        }).collect::<Result<Vec<(Vec<u8>, &_, &_)>, cbor_event::Error>>()?;
                        if force_canonical {
                            key_order.sort_by(|(lhs_bytes, _, _), (rhs_bytes, _, _)| {
                                match lhs_bytes.len().cmp(&rhs_bytes.len()) {
                                    std::cmp::Ordering::Equal => lhs_bytes.cmp(&rhs_bytes),
                                    diff_ord => diff_ord,
                                }
                            });
                        }
                        for (key_bytes, key, value) in key_order {
                            serializer.write_raw_bytes(&key_bytes)?;
                            value.serialize(serializer, force_canonical)?;
                        }
                        data_value_encoding.end(serializer, force_canonical)?;
                    }
                    self.data_encoding.end(serializer, force_canonical)?;
                }
                1 => {
                    serializer.write_text_sz(&"version", self.version_key_encoding.to_str_len_sz("version".len() as u64, force_canonical))?;
                    serializer.write_unsigned_integer_sz(2u64, fit_sz(2u64, self.version_encoding, force_canonical))?;
                }
                _ => unreachable!()
            };
        }
        self.encoding.end(serializer, force_canonical)
    }
}

impl Deserialize for LabelMetadata {
    fn deserialize<R: BufRead + Seek>(raw: &mut Deserializer<R>) -> Result<Self, DeserializeError> {
        (|| -> Result<_, DeserializeError> {
            let len = raw.map_sz()?;
            let encoding = len.into();
            let mut read_len = CBORReadLen::new(len);
            read_len.read_elems(2)?;
            let mut orig_deser_order = Vec::new();
            let mut data_encoding = LenEncoding::default();
            let mut data_key_encodings = BTreeMap::new();
            let mut data_value_encodings = BTreeMap::new();
            let mut data_key_encoding = StringEncoding::default();
            let mut data = None;
            let mut version_encoding = None;
            let mut version_key_encoding = StringEncoding::default();
            let mut version_present = false;
            let mut read = 0;
            while match len { cbor_event::LenSz::Len(n, _) => read < n as usize, cbor_event::LenSz::Indefinite => true, } {
                match raw.cbor_type()? {
                    CBORType::UnsignedInteger => match raw.unsigned_integer_sz()? {
                        (unknown_key, _enc) => return Err(DeserializeFailure::UnknownKey(Key::Uint(unknown_key)).into()),
                    },
                    CBORType::Text => {
                        let (text_key, key_enc) = raw.text_sz()?;
                        match text_key.as_str() {
                            "data" =>  {
                                if data.is_some() {
                                    return Err(DeserializeFailure::DuplicateKey(Key::Str("data".into())).into());
                                }
                                let (tmp_data, tmp_data_encoding, tmp_data_key_encodings, tmp_data_value_encodings) = (|| -> Result<_, DeserializeError> {
                                    let mut data_table = LinkedHashMap::new();
                                    let data_len = raw.map_sz()?;
                                    let data_encoding = data_len.into();
                                    let mut data_key_encodings = BTreeMap::new();
                                    let mut data_value_encodings = BTreeMap::new();
                                    while match data_len { cbor_event::LenSz::Len(n, _) => data_table.len() < n as usize, cbor_event::LenSz::Indefinite => true, } {
                                        if raw.cbor_type()? == CBORType::Special {
                                            assert_eq!(raw.special()?, CBORSpecial::Break);
                                            break;
                                        }
                                        let (data_key, data_key_encoding) = raw.bytes_sz().map(|(bytes, enc)| (bytes, StringEncoding::from(enc)))?;
                                        let mut data_value_table = LinkedHashMap::new();
                                        let data_value_len = raw.map_sz()?;
                                        let data_value_encoding = data_value_len.into();
                                        let mut data_value_key_encodings = BTreeMap::new();
                                        while match data_value_len { cbor_event::LenSz::Len(n, _) => data_value_table.len() < n as usize, cbor_event::LenSz::Indefinite => true, } {
                                            if raw.cbor_type()? == CBORType::Special {
                                                assert_eq!(raw.special()?, CBORSpecial::Break);
                                                break;
                                            }
                                            let (data_value_key, data_value_key_encoding) = raw.bytes_sz().map(|(bytes, enc)| (bytes, StringEncoding::from(enc)))?;
                                            let data_value_value = MetadataDetails::deserialize(raw)?;
                                            if data_value_table.insert(data_value_key.clone(), data_value_value).is_some() {
                                                return Err(DeserializeFailure::DuplicateKey(Key::Str(String::from("some complicated/unsupported type"))).into());
                                            }
                                            data_value_key_encodings.insert(data_value_key.clone(), data_value_key_encoding);
                                        }
                                        let (data_value, data_value_encoding, data_value_key_encodings) = (data_value_table, data_value_encoding, data_value_key_encodings);
                                        if data_table.insert(data_key.clone(), data_value).is_some() {
                                            return Err(DeserializeFailure::DuplicateKey(Key::Str(String::from("some complicated/unsupported type"))).into());
                                        }
                                        data_key_encodings.insert(data_key.clone(), data_key_encoding);
                                        data_value_encodings.insert(data_key.clone(), (data_value_encoding, data_value_key_encodings));
                                    }
                                    Ok((data_table, data_encoding, data_key_encodings, data_value_encodings))
                                })().map_err(|e| e.annotate("data"))?;
                                data = Some(tmp_data);
                                data_encoding = tmp_data_encoding;
                                data_key_encodings = tmp_data_key_encodings;
                                data_value_encodings = tmp_data_value_encodings;
                                data_key_encoding = StringEncoding::from(key_enc);
                                orig_deser_order.push(0);
                            },
                            "version" =>  {
                                if version_present {
                                    return Err(DeserializeFailure::DuplicateKey(Key::Str("version".into())).into());
                                }
                                let tmp_version_encoding = (|| -> Result<_, DeserializeError> {
                                    let (version_value, version_encoding) = raw.unsigned_integer_sz()?;
                                    if version_value != 2 {
                                        return Err(DeserializeFailure::FixedValueMismatch{ found: Key::Uint(version_value), expected: Key::Uint(2) }.into());
                                    }
                                    Ok(Some(version_encoding))
                                })().map_err(|e| e.annotate("version"))?;
                                version_present = true;
                                version_encoding = tmp_version_encoding;
                                version_key_encoding = StringEncoding::from(key_enc);
                                orig_deser_order.push(1);
                            },
                            unknown_key => return Err(DeserializeFailure::UnknownKey(Key::Str(unknown_key.to_owned())).into()),
                        }
                    },
                    CBORType::Special => match len {
                        cbor_event::LenSz::Len(_, _) => return Err(DeserializeFailure::BreakInDefiniteLen.into()),
                        cbor_event::LenSz::Indefinite => match raw.special()? {
                            CBORSpecial::Break => break,
                            _ => return Err(DeserializeFailure::EndingBreakMissing.into()),
                        },
                    },
                    other_type => return Err(DeserializeFailure::UnexpectedKeyType(other_type).into()),
                }
                read += 1;
            }
            let data = match data {
                Some(x) => x,
                None => return Err(DeserializeFailure::MandatoryFieldMissing(Key::Str(String::from("data"))).into()),
            };
            if !version_present {
                return Err(DeserializeFailure::MandatoryFieldMissing(Key::Str(String::from("version"))).into());
            }
            ();
            Ok(Self {
                data,
                data_encoding,
                data_key_encodings,
                data_value_encodings,
                data_key_encoding,
                version_encoding,
                version_key_encoding,
                encoding,
                orig_deser_order: Some(orig_deser_order),
            })
        })().map_err(|e| e.annotate("LabelMetadata"))
    }
}

impl Serialize for Metadata {
    fn serialize<'se, W: Write>(&self, serializer: &'se mut Serializer<W>, force_canonical: bool) -> cbor_event::Result<&'se mut Serializer<W>> {
        serializer.write_map_sz(self.encoding.to_len_sz(1, force_canonical))?;
        let deser_order = if !force_canonical && self.orig_deser_order.as_ref().map(|v| v.len() == 1).unwrap_or(false) {
            self.orig_deser_order.clone().unwrap()
        }
        else {
            vec![0]
        };
        for field_index in deser_order {
            match field_index {
                0 => {
                    serializer.write_unsigned_integer_sz(721u64, fit_sz(721u64, self.key_721_key_encoding, force_canonical))?;
                    self.key_721.serialize(serializer, force_canonical)?;
                }
                _ => unreachable!()
            };
        }
        self.encoding.end(serializer, force_canonical)
    }
}

impl Deserialize for Metadata {
    fn deserialize<R: BufRead + Seek>(raw: &mut Deserializer<R>) -> Result<Self, DeserializeError> {
        (|| -> Result<_, DeserializeError> {
            let len = raw.map_sz()?;
            let encoding = len.into();
            let mut read_len = CBORReadLen::new(len);
            read_len.read_elems(1)?;
            let mut orig_deser_order = Vec::new();
            let mut key_721_key_encoding = None;
            let mut key_721 = None;
            let mut read = 0;
            while match len { cbor_event::LenSz::Len(n, _) => read < n as usize, cbor_event::LenSz::Indefinite => true, } {
                match raw.cbor_type()? {
                    CBORType::UnsignedInteger => match raw.unsigned_integer_sz()? {
                        (721, key_enc) =>  {
                            if key_721.is_some() {
                                return Err(DeserializeFailure::DuplicateKey(Key::Uint(721)).into());
                            }
                            let tmp_key_721 = (|| -> Result<_, DeserializeError> {
                                Ok(LabelMetadata::deserialize(raw)?)
                            })().map_err(|e| e.annotate("key_721"))?;
                            key_721 = Some(tmp_key_721);
                            key_721_key_encoding = Some(key_enc);
                            orig_deser_order.push(0);
                        },
                        (unknown_key, _enc) => return Err(DeserializeFailure::UnknownKey(Key::Uint(unknown_key)).into()),
                    },
                    CBORType::Text => {
                        let (text_key, key_enc) = raw.text_sz()?;
                        match text_key.as_str() {
                            unknown_key => return Err(DeserializeFailure::UnknownKey(Key::Str(unknown_key.to_owned())).into()),
                        }
                    },
                    CBORType::Special => match len {
                        cbor_event::LenSz::Len(_, _) => return Err(DeserializeFailure::BreakInDefiniteLen.into()),
                        cbor_event::LenSz::Indefinite => match raw.special()? {
                            CBORSpecial::Break => break,
                            _ => return Err(DeserializeFailure::EndingBreakMissing.into()),
                        },
                    },
                    other_type => return Err(DeserializeFailure::UnexpectedKeyType(other_type).into()),
                }
                read += 1;
            }
            let key_721 = match key_721 {
                Some(x) => x,
                None => return Err(DeserializeFailure::MandatoryFieldMissing(Key::Uint(721)).into()),
            };
            ();
            Ok(Self {
                key_721,
                key_721_key_encoding,
                encoding,
                orig_deser_order: Some(orig_deser_order),
            })
        })().map_err(|e| e.annotate("Metadata"))
    }
}

impl Serialize for MetadataDetails {
    fn serialize<'se, W: Write>(&self, serializer: &'se mut Serializer<W>, force_canonical: bool) -> cbor_event::Result<&'se mut Serializer<W>> {
        serializer.write_map_sz(self.encoding.to_len_sz(2 + match &self.media_type { Some(_) => 1, None => 0 } + match &self.description { Some(_) => 1, None => 0 } + match &self.files { Some(_) => 1, None => 0 }, force_canonical))?;
        let deser_order = if !force_canonical && self.orig_deser_order.as_ref().map(|v| v.len() == 2 + match &self.media_type { Some(_) => 1, None => 0 } + match &self.description { Some(_) => 1, None => 0 } + match &self.files { Some(_) => 1, None => 0 }).unwrap_or(false) {
            self.orig_deser_order.clone().unwrap()
        }
        else {
            vec![0,4,1,2,3]
        };
        for field_index in deser_order {
            match field_index {
                0 => {
                    serializer.write_text_sz(&"name", self.name_key_encoding.to_str_len_sz("name".len() as u64, force_canonical))?;
                    self.name.serialize(serializer, force_canonical)?;
                }
                4 => if let Some(field) = &self.files {
                    serializer.write_text_sz(&"files", self.files_key_encoding.to_str_len_sz("files".len() as u64, force_canonical))?;
                    serializer.write_array_sz(self.files_encoding.to_len_sz(field.len() as u64, force_canonical))?;
                    for element in field.iter() {
                        element.serialize(serializer, force_canonical)?;
                    }
                    self.files_encoding.end(serializer, force_canonical)?;
                }
                1 => {
                    serializer.write_text_sz(&"image", self.image_key_encoding.to_str_len_sz("image".len() as u64, force_canonical))?;
                    self.image.serialize(serializer, force_canonical)?;
                }
                2 => if let Some(field) = &self.media_type {
                    serializer.write_text_sz(&"mediaType", self.media_type_key_encoding.to_str_len_sz("mediaType".len() as u64, force_canonical))?;
                    field.serialize(serializer, force_canonical)?;
                }
                3 => if let Some(field) = &self.description {
                    serializer.write_text_sz(&"description", self.description_key_encoding.to_str_len_sz("description".len() as u64, force_canonical))?;
                    field.serialize(serializer, force_canonical)?;
                }
                _ => unreachable!()
            };
        }
        self.encoding.end(serializer, force_canonical)
    }
}

impl Deserialize for MetadataDetails {
    fn deserialize<R: BufRead + Seek>(raw: &mut Deserializer<R>) -> Result<Self, DeserializeError> {
        (|| -> Result<_, DeserializeError> {
            let len = raw.map_sz()?;
            let encoding = len.into();
            let mut read_len = CBORReadLen::new(len);
            read_len.read_elems(2)?;
            let mut orig_deser_order = Vec::new();
            let mut name_key_encoding = StringEncoding::default();
            let mut name = None;
            let mut files_encoding = LenEncoding::default();
            let mut files_key_encoding = StringEncoding::default();
            let mut files = None;
            let mut image_key_encoding = StringEncoding::default();
            let mut image = None;
            let mut media_type_key_encoding = StringEncoding::default();
            let mut media_type = None;
            let mut description_key_encoding = StringEncoding::default();
            let mut description = None;
            let mut read = 0;
            while match len { cbor_event::LenSz::Len(n, _) => read < n as usize, cbor_event::LenSz::Indefinite => true, } {
                match raw.cbor_type()? {
                    CBORType::UnsignedInteger => match raw.unsigned_integer_sz()? {
                        (unknown_key, _enc) => return Err(DeserializeFailure::UnknownKey(Key::Uint(unknown_key)).into()),
                    },
                    CBORType::Text => {
                        let (text_key, key_enc) = raw.text_sz()?;
                        match text_key.as_str() {
                            "name" =>  {
                                if name.is_some() {
                                    return Err(DeserializeFailure::DuplicateKey(Key::Str("name".into())).into());
                                }
                                let tmp_name = (|| -> Result<_, DeserializeError> {
                                    Ok(String64::deserialize(raw)?)
                                })().map_err(|e| e.annotate("name"))?;
                                name = Some(tmp_name);
                                name_key_encoding = StringEncoding::from(key_enc);
                                orig_deser_order.push(0);
                            },
                            "files" =>  {
                                if files.is_some() {
                                    return Err(DeserializeFailure::DuplicateKey(Key::Str("files".into())).into());
                                }
                                let (tmp_files, tmp_files_encoding) = (|| -> Result<_, DeserializeError> {
                                    read_len.read_elems(1)?;
                                    let mut files_arr = Vec::new();
                                    let len = raw.array_sz()?;
                                    let files_encoding = len.into();
                                    while match len { cbor_event::LenSz::Len(n, _) => files_arr.len() < n as usize, cbor_event::LenSz::Indefinite => true, } {
                                        if raw.cbor_type()? == CBORType::Special {
                                            assert_eq!(raw.special()?, CBORSpecial::Break);
                                            break;
                                        }
                                        files_arr.push(FilesDetails::deserialize(raw)?);
                                    }
                                    Ok((files_arr, files_encoding))
                                })().map_err(|e| e.annotate("files"))?;
                                files = Some(tmp_files);
                                files_encoding = tmp_files_encoding;
                                files_key_encoding = StringEncoding::from(key_enc);
                                orig_deser_order.push(4);
                            },
                            "image" =>  {
                                if image.is_some() {
                                    return Err(DeserializeFailure::DuplicateKey(Key::Str("image".into())).into());
                                }
                                let tmp_image = (|| -> Result<_, DeserializeError> {
                                    Ok(String64OrArrString64::deserialize(raw)?)
                                })().map_err(|e| e.annotate("image"))?;
                                image = Some(tmp_image);
                                image_key_encoding = StringEncoding::from(key_enc);
                                orig_deser_order.push(1);
                            },
                            "mediaType" =>  {
                                if media_type.is_some() {
                                    return Err(DeserializeFailure::DuplicateKey(Key::Str("mediaType".into())).into());
                                }
                                let tmp_media_type = (|| -> Result<_, DeserializeError> {
                                    read_len.read_elems(1)?;
                                    Ok(String64::deserialize(raw)?)
                                })().map_err(|e| e.annotate("media_type"))?;
                                media_type = Some(tmp_media_type);
                                media_type_key_encoding = StringEncoding::from(key_enc);
                                orig_deser_order.push(2);
                            },
                            "description" =>  {
                                if description.is_some() {
                                    return Err(DeserializeFailure::DuplicateKey(Key::Str("description".into())).into());
                                }
                                let tmp_description = (|| -> Result<_, DeserializeError> {
                                    read_len.read_elems(1)?;
                                    Ok(String64OrArrString64::deserialize(raw)?)
                                })().map_err(|e| e.annotate("description"))?;
                                description = Some(tmp_description);
                                description_key_encoding = StringEncoding::from(key_enc);
                                orig_deser_order.push(3);
                            },
                            unknown_key => return Err(DeserializeFailure::UnknownKey(Key::Str(unknown_key.to_owned())).into()),
                        }
                    },
                    CBORType::Special => match len {
                        cbor_event::LenSz::Len(_, _) => return Err(DeserializeFailure::BreakInDefiniteLen.into()),
                        cbor_event::LenSz::Indefinite => match raw.special()? {
                            CBORSpecial::Break => break,
                            _ => return Err(DeserializeFailure::EndingBreakMissing.into()),
                        },
                    },
                    other_type => return Err(DeserializeFailure::UnexpectedKeyType(other_type).into()),
                }
                read += 1;
            }
            let name = match name {
                Some(x) => x,
                None => return Err(DeserializeFailure::MandatoryFieldMissing(Key::Str(String::from("name"))).into()),
            };
            let image = match image {
                Some(x) => x,
                None => return Err(DeserializeFailure::MandatoryFieldMissing(Key::Str(String::from("image"))).into()),
            };
            read_len.finish()?;
            Ok(Self {
                name,
                name_key_encoding,
                image,
                image_key_encoding,
                media_type,
                media_type_key_encoding,
                description,
                description_key_encoding,
                files,
                files_encoding,
                files_key_encoding,
                encoding,
                orig_deser_order: Some(orig_deser_order),
            })
        })().map_err(|e| e.annotate("MetadataDetails"))
    }
}

impl Serialize for String64 {
    fn serialize<'se, W: Write>(&self, serializer: &'se mut Serializer<W>, force_canonical: bool) -> cbor_event::Result<&'se mut Serializer<W>> {
        serializer.write_text_sz(&self.inner, self.inner_encoding.to_str_len_sz(self.inner.len() as u64, force_canonical))
    }
}

impl Deserialize for String64 {
    fn deserialize<R: BufRead + Seek>(raw: &mut Deserializer<R>) -> Result<Self, DeserializeError> {
        let (inner, inner_encoding) = raw.text_sz().map(|(s, enc)| (s, StringEncoding::from(enc)))?;
        if inner.len() > 64 {
            return Err(DeserializeError::new("String64", DeserializeFailure::RangeCheck{ found: inner.len(), min: Some(0), max: Some(64) }));
        }
        Ok(Self {
            inner,
            inner_encoding,
        })
    }
}

impl Serialize for String64OrArrString64 {
    fn serialize<'se, W: Write>(&self, serializer: &'se mut Serializer<W>, force_canonical: bool) -> cbor_event::Result<&'se mut Serializer<W>> {
        match self {
            String64OrArrString64::String64(x) => {
                x.serialize(serializer, force_canonical)
            },
            String64OrArrString64::ArrString64(x, x_encoding) => {
                serializer.write_array_sz(x_encoding.to_len_sz(x.len() as u64, force_canonical))?;
                for element in x.iter() {
                    element.serialize(serializer, force_canonical)?;
                }
                x_encoding.end(serializer, force_canonical)
            },
        }
    }
}

impl Deserialize for String64OrArrString64 {
    fn deserialize<R: BufRead + Seek>(raw: &mut Deserializer<R>) -> Result<Self, DeserializeError> {
        (|| -> Result<_, DeserializeError> {
            let initial_position = raw.as_mut_ref().seek(SeekFrom::Current(0)).unwrap();
            match (|raw: &mut Deserializer<_>| -> Result<_, DeserializeError> {
                Ok(String64::deserialize(raw)?)
            })(raw)
            {
                Ok((x)) => return Ok(String64OrArrString64::String64(x)),
                Err(_) => raw.as_mut_ref().seek(SeekFrom::Start(initial_position)).unwrap(),
            };
            match (|raw: &mut Deserializer<_>| -> Result<_, DeserializeError> {
                let mut arr_string64_arr = Vec::new();
                let len = raw.array_sz()?;
                let arr_string64_encoding = len.into();
                while match len { cbor_event::LenSz::Len(n, _) => arr_string64_arr.len() < n as usize, cbor_event::LenSz::Indefinite => true, } {
                    if raw.cbor_type()? == CBORType::Special {
                        assert_eq!(raw.special()?, CBORSpecial::Break);
                        break;
                    }
                    arr_string64_arr.push(String64::deserialize(raw)?);
                }
                Ok((arr_string64_arr, arr_string64_encoding))
            })(raw)
            {
                Ok((x, x_encoding)) => return Ok(String64OrArrString64::ArrString64(x, x_encoding)),
                Err(_) => raw.as_mut_ref().seek(SeekFrom::Start(initial_position)).unwrap(),
            };
            Err(DeserializeError::new("String64OrArrString64", DeserializeFailure::NoVariantMatched.into()))
        })().map_err(|e| e.annotate("String64OrArrString64"))
    }
}