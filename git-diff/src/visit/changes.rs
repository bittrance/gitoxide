use crate::visit;
use crate::visit::record::{Change, PathComponent, PathComponentUpdateMode};
use git_hash::{oid, ObjectId};
use git_object::{immutable, tree};
use quick_error::quick_error;

static EMPTY_TREE: immutable::Tree<'static> = immutable::Tree::empty();

quick_error! {
    #[derive(Debug)]
    pub enum Error {
        NotFound(oid: ObjectId) {
            display("The object {} referenced by the tree was not found in the database", oid)
        }
        Cancelled {
            display("The delegate cancelled the operation")
        }
    }
}

impl<'a> visit::Changes<'a> {
    /// Returns the changes that need to be applied to `self` to get `other`.
    ///
    /// # Notes
    ///
    /// * Tree entries are expected to be ordered using [`tree-entry-comparison`][git_cmp_c] (the same [in Rust][git_cmp_rs])
    ///
    /// [git_cmp_c]: https://github.com/git/git/blob/311531c9de557d25ac087c1637818bd2aad6eb3a/tree-diff.c#L49:L65
    /// [git_cmp_rs]: https://github.com/Byron/gitoxide/blob/a4d5f99c8dc99bf814790928a3bf9649cd99486b/git-object/src/mutable/tree.rs#L52-L55
    ///
    pub fn to_obtain_tree<LocateFn>(
        &self,
        other: &git_object::immutable::Tree<'_>,
        _state: &mut visit::State,
        _locate: LocateFn,
        delegate: &mut impl visit::Record,
    ) -> Result<(), Error>
    where
        LocateFn: for<'b> FnMut(&oid, &'b mut Vec<u8>) -> Option<immutable::Object<'b>>,
    {
        let lhs = *self.0.as_ref().unwrap_or(&&EMPTY_TREE);
        let rhs = other;

        let mut path_id = 0;
        let mut lhs_entries = lhs.entries.iter();
        let mut rhs_entries = rhs.entries.iter();

        loop {
            match (lhs_entries.next(), rhs_entries.next()) {
                (None, None) => break Ok(()),
                (Some(lhs), Some(rhs)) => {
                    use std::cmp::Ordering::*;
                    match lhs.filename.cmp(rhs.filename) {
                        Equal => {
                            use tree::EntryMode::*;
                            if lhs.oid != rhs.oid || lhs.mode != rhs.mode {
                                delegate.update_path_component(
                                    PathComponent::new(lhs.filename, &mut path_id),
                                    PathComponentUpdateMode::Replace,
                                );
                                let record_result = if (lhs.mode.is_no_tree() && rhs.mode.is_tree())
                                    || (rhs.mode.is_tree() && rhs.mode.is_no_tree())
                                {
                                    delegate.record(Change::Deletion {
                                        entry_mode: lhs.mode,
                                        oid: lhs.oid.to_owned(),
                                        path_id,
                                    })
                                } else {
                                    delegate.record(Change::Modification {
                                        previous_entry_mode: lhs.mode,
                                        previous_oid: lhs.oid.to_owned(),
                                        entry_mode: rhs.mode,
                                        oid: rhs.oid.to_owned(),
                                        path_id,
                                    })
                                };
                                if record_result.cancelled() {
                                    break Err(Error::Cancelled);
                                }
                            }
                            match (lhs.mode, rhs.mode) {
                                (Tree, Tree) => todo!("recurse tree|tree"),
                                (lhs, Tree) if !lhs.is_tree() => todo!("recurse non-tree|tree"),
                                (Tree, rhs) if !rhs.is_tree() => todo!("recurse tree|non-tree"),
                                _both_are_not_trees => {}
                            }
                        }
                        Less => todo!("entry compares less - catch up"),
                        Greater => todo!("entry compares more - let the other side catch up"),
                    }
                }
                (Some(lhs), None) => {
                    delegate.update_path_component(
                        PathComponent::new(lhs.filename, &mut path_id),
                        PathComponentUpdateMode::Replace,
                    );
                    if delegate
                        .record(Change::Deletion {
                            entry_mode: lhs.mode,
                            oid: lhs.oid.to_owned(),
                            path_id,
                        })
                        .cancelled()
                    {
                        break Err(Error::Cancelled);
                    }
                }
                (None, Some(rhs)) => {
                    delegate.update_path_component(
                        PathComponent::new(rhs.filename, &mut path_id),
                        PathComponentUpdateMode::Replace,
                    );
                    if delegate
                        .record(Change::Addition {
                            entry_mode: rhs.mode,
                            oid: rhs.oid.to_owned(),
                            path_id,
                        })
                        .cancelled()
                    {
                        break Err(Error::Cancelled);
                    }
                }
            }
        }
    }
}
