use bstr::BStr;

/// TODO
pub trait Key<'a> {
    /// TODO
    fn name(&self) -> &'a str;
    /// TODO
    fn section_name(&self) -> &'a str;
    /// TODO
    fn subsection_name(&self) -> Option<&'a BStr>;
}
