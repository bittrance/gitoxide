use bstr::{BStr, BString, ByteSlice};

use crate::traits::Key;

impl<'a> Key<'a> for &'a String {
    fn name(&self) -> &'a str {
        self.rsplit_once('.').map_or(self, |(_, name)| name)
    }

    fn section_name(&self) -> &'a str {
        self.split_once('.').map_or("", |(section, _)| section)
    }

    fn subsection_name(&self) -> Option<&'a BStr> {
        self.rsplit_once('.')
            .and_then(|(section, _)| section.split_once('.').map(|(_, subsection)| subsection.into()))
    }
}

impl<'a> Key<'a> for &'a str {
    fn name(&self) -> &'a str {
        self.rsplit_once('.').map_or(self, |(_, name)| name)
    }

    fn section_name(&self) -> &'a str {
        self.split_once('.').map_or("", |(section, _)| section)
    }

    fn subsection_name(&self) -> Option<&'a BStr> {
        self.rsplit_once('.')
            .and_then(|(section, _)| section.split_once('.').map(|(_, subsection)| subsection.into()))
    }
}

impl<'a> Key<'a> for &'a BString {
    fn name(&self) -> &'a str {
        self.rsplit(|b| *b == b'.').next().unwrap().to_str().unwrap()
    }

    fn section_name(&self) -> &'a str {
        let mut parts = self.split(|b| *b == b'.');
        let section = parts.next();
        if parts.next().is_some() {
            section.unwrap().to_str().unwrap()
        } else {
            ""
        }
    }

    fn subsection_name(&self) -> Option<&'a BStr> {
        let subsection_or_key = self.splitn(2, |b| *b == b'.').nth(1)?.as_bstr();
        let mut parts = subsection_or_key.rsplitn(2, |b| *b == b'.');
        let (key, subsection) = (parts.next(), parts.next());
        if key.is_some() {
            subsection.map(bstr::ByteSlice::as_bstr)
        } else {
            None
        }
    }
}

impl<'a> Key<'a> for &'a BStr {
    fn name(&self) -> &'a str {
        self.rsplit(|b| *b == b'.').next().unwrap().to_str().unwrap()
    }

    fn section_name(&self) -> &'a str {
        let mut parts = self.split(|b| *b == b'.');
        let section = parts.next();
        if parts.next().is_some() {
            section.unwrap().to_str().unwrap()
        } else {
            ""
        }
    }

    fn subsection_name(&self) -> Option<&'a BStr> {
        let subsection_or_key = self.splitn(2, |b| *b == b'.').nth(1)?.as_bstr();
        let mut parts = subsection_or_key.rsplitn(2, |b| *b == b'.');
        let (key, subsection) = (parts.next(), parts.next());
        if key.is_some() {
            subsection.map(bstr::ByteSlice::as_bstr)
        } else {
            None
        }
    }
}
