use std::{borrow::Cow, convert::TryFrom};

use bstr::BStr;

use crate::{file::MetadataFilter, value, File, Key};

/// Comfortable API for accessing values
impl<'event> File<'event> {
    /// Like [`value()`][File::value()], but returning `None` if the string wasn't found.
    ///
    /// As strings perform no conversions, this will never fail.
    pub fn string<'a>(&self, key: impl Key<'a>) -> Option<Cow<'_, BStr>> {
        self.string_filter(key, &mut |_| true)
    }

    /// Like [`string()`][File::string()], but suitable for statically known `key`s like `remote.origin.url`.
    pub fn string_by_key<'a>(&self, key: impl Key<'a>) -> Option<Cow<'_, BStr>> {
        self.string_filter_by_key(key, &mut |_| true)
    }

    /// Like [`string()`][File::string()], but the section containing the returned value must pass `filter` as well.
    pub fn string_filter<'a>(&self, key: impl Key<'a>, filter: &mut MetadataFilter) -> Option<Cow<'_, BStr>> {
        self.raw_value_filter(key, filter).ok()
    }

    /// Like [`string_filter()`][File::string_filter()], but suitable for statically known `key`s like `remote.origin.url`.
    pub fn string_filter_by_key<'a>(&self, key: impl Key<'a>, filter: &mut MetadataFilter) -> Option<Cow<'_, BStr>> {
        self.raw_value_filter(key, filter).ok()
    }

    /// Like [`value()`][File::value()], but returning `None` if the path wasn't found.
    ///
    /// Note that this path is not vetted and should only point to resources which can't be used
    /// to pose a security risk. Prefer using [`path_filter()`][File::path_filter()] instead.
    ///
    /// As paths perform no conversions, this will never fail.
    pub fn path<'a>(&self, key: impl Key<'a>) -> Option<crate::Path<'_>> {
        self.path_filter(key, &mut |_| true)
    }

    /// Like [`path()`][File::path()], but suitable for statically known `key`s like `remote.origin.url`.
    pub fn path_by_key<'a>(&self, key: impl Key<'a>) -> Option<crate::Path<'_>> {
        self.path_filter_by_key(key, &mut |_| true)
    }

    /// Like [`path()`][File::path()], but the section containing the returned value must pass `filter` as well.
    ///
    /// This should be the preferred way of accessing paths as those from untrusted
    /// locations can be
    ///
    /// As paths perform no conversions, this will never fail.
    pub fn path_filter<'a>(&self, key: impl Key<'a>, filter: &mut MetadataFilter) -> Option<crate::Path<'_>> {
        self.raw_value_filter(key, filter).ok().map(crate::Path::from)
    }

    /// Like [`path_filter()`][File::path_filter()], but suitable for statically known `key`s like `remote.origin.url`.
    pub fn path_filter_by_key<'a>(&self, key: impl Key<'a>, filter: &mut MetadataFilter) -> Option<crate::Path<'_>> {
        self.path_filter(key, filter)
    }

    /// Like [`value()`][File::value()], but returning `None` if the boolean value wasn't found.
    pub fn boolean<'a>(&self, key: impl Key<'a>) -> Option<Result<bool, value::Error>> {
        self.boolean_filter(key, &mut |_| true)
    }

    /// Like [`boolean()`][File::boolean()], but suitable for statically known `key`s like `remote.origin.url`.
    pub fn boolean_by_key<'a>(&self, key: impl Key<'a>) -> Option<Result<bool, value::Error>> {
        self.boolean_filter_by_key(key, &mut |_| true)
    }

    /// Like [`boolean()`][File::boolean()], but the section containing the returned value must pass `filter` as well.
    pub fn boolean_filter<'a>(
        &self,
        key: impl Key<'a>,
        filter: &mut MetadataFilter,
    ) -> Option<Result<bool, value::Error>> {
        let section_ids = self
            .section_ids_by_name_and_subname(key.section_name(), key.subsection_name())
            .ok()?;
        for section_id in section_ids.rev() {
            let section = self.sections.get(&section_id).expect("known section id");
            if !filter(section.meta()) {
                continue;
            }
            match section.value_implicit(key.name()) {
                Some(Some(v)) => return Some(crate::Boolean::try_from(v).map(Into::into)),
                Some(None) => return Some(Ok(true)),
                None => continue,
            }
        }
        None
    }

    /// Like [`boolean_filter()`][File::boolean_filter()], but suitable for statically known `key`s like `remote.origin.url`.
    pub fn boolean_filter_by_key<'a>(
        &self,
        key: impl Key<'a>,
        filter: &mut MetadataFilter,
    ) -> Option<Result<bool, value::Error>> {
        self.boolean_filter(key, filter)
    }

    /// Like [`value()`][File::value()], but returning an `Option` if the integer wasn't found.
    pub fn integer<'a>(&self, key: impl Key<'a>) -> Option<Result<i64, value::Error>> {
        self.integer_filter(key, &mut |_| true)
    }

    /// Like [`integer()`][File::integer()], but suitable for statically known `key`s like `remote.origin.url`.
    pub fn integer_by_key<'a>(&self, key: impl Key<'a>) -> Option<Result<i64, value::Error>> {
        self.integer_filter_by_key(key, &mut |_| true)
    }

    /// Like [`integer()`][File::integer()], but the section containing the returned value must pass `filter` as well.
    pub fn integer_filter<'a>(
        &self,
        key: impl Key<'a>,
        filter: &mut MetadataFilter,
    ) -> Option<Result<i64, value::Error>> {
        let int = self.raw_value_filter(key, filter).ok()?;
        Some(crate::Integer::try_from(int.as_ref()).and_then(|b| {
            b.to_decimal()
                .ok_or_else(|| value::Error::new("Integer overflow", int.into_owned()))
        }))
    }

    /// Like [`integer_filter()`][File::integer_filter()], but suitable for statically known `key`s like `remote.origin.url`.
    pub fn integer_filter_by_key<'a>(
        &self,
        key: impl Key<'a>,
        filter: &mut MetadataFilter,
    ) -> Option<Result<i64, value::Error>> {
        self.integer_filter(key, filter)
    }

    /// Similar to [`values(…)`][File::values()] but returning strings if at least one of them was found.
    pub fn strings<'a>(&self, key: impl Key<'a>) -> Option<Vec<Cow<'_, BStr>>> {
        self.raw_values(key).ok()
    }

    /// Like [`strings()`][File::strings()], but suitable for statically known `key`s like `remote.origin.url`.
    pub fn strings_by_key<'a>(&self, key: impl Key<'a>) -> Option<Vec<Cow<'_, BStr>>> {
        self.strings(key)
    }

    /// Similar to [`strings(…)`][File::strings()], but all values are in sections that passed `filter`.
    pub fn strings_filter<'a>(&self, key: impl Key<'a>, filter: &mut MetadataFilter) -> Option<Vec<Cow<'_, BStr>>> {
        self.raw_values_filter(key, filter).ok()
    }

    /// Like [`strings_filter()`][File::strings_filter()], but suitable for statically known `key`s like `remote.origin.url`.
    pub fn strings_filter_by_key<'a>(
        &self,
        key: impl Key<'a>,
        filter: &mut MetadataFilter,
    ) -> Option<Vec<Cow<'_, BStr>>> {
        self.strings_filter(key, filter)
    }

    /// Similar to [`values(…)`][File::values()] but returning integers if at least one of them was found
    /// and if none of them overflows.
    pub fn integers<'a>(&self, key: impl Key<'a>) -> Option<Result<Vec<i64>, value::Error>> {
        self.integers_filter(key, &mut |_| true)
    }

    /// Like [`integers()`][File::integers()], but suitable for statically known `key`s like `remote.origin.url`.
    pub fn integers_by_key<'a>(&self, key: impl Key<'a>) -> Option<Result<Vec<i64>, value::Error>> {
        self.integers_filter_by_key(key, &mut |_| true)
    }

    /// Similar to [`integers(…)`][File::integers()] but all integers are in sections that passed `filter`
    /// and that are not overflowing.
    pub fn integers_filter<'a>(
        &self,
        key: impl Key<'a>,
        filter: &mut MetadataFilter,
    ) -> Option<Result<Vec<i64>, value::Error>> {
        self.raw_values_filter(key, filter).ok().map(|values| {
            values
                .into_iter()
                .map(|v| {
                    crate::Integer::try_from(v.as_ref()).and_then(|int| {
                        int.to_decimal()
                            .ok_or_else(|| value::Error::new("Integer overflow", v.into_owned()))
                    })
                })
                .collect()
        })
    }

    /// Like [`integers_filter()`][File::integers_filter()], but suitable for statically known `key`s like `remote.origin.url`.
    pub fn integers_filter_by_key<'a>(
        &self,
        key: impl Key<'a>,
        filter: &mut MetadataFilter,
    ) -> Option<Result<Vec<i64>, value::Error>> {
        self.integers_filter(key, filter)
    }
}
