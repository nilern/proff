mod util;
mod descriptor;
mod layout;
mod arena;
mod block;
mod mark_n_sweep;

use std::ptr::Shared;

pub use self::layout::GSize;
pub use self::util::{Uninitialized, Initializable, start_init};
pub use self::mark_n_sweep::Generation;

/// Managed heap value.
pub trait Object {
    /// The `ObjectRef` pointing to and contained in fields of `Self`.
    type ORef: ObjectRef;
    /// Iterator over pointers to `ValueRef` fields of `Self`.
    type RefIter: Iterator<Item=Shared<Self::ORef>>;

    /// Allocated size of `self` in granules.
    fn gsize(&self) -> GSize;

    /// Get the `Self::RefIter` for `self`.
    fn obj_refs(&self) -> Self::RefIter;
}

/// Object reference.
pub trait ObjectRef: Copy {
    /// The `Object` (possibly) pointed to by `Self`.
    type Obj: Object;

    /// Get a pointer to the contained `Object` if one exists.
    fn ptr(self) -> Option<Shared<Self::Obj>>;

    /// Does `self` contain a pointer to a value containing `Self` fields.
    fn is_pointy(self) -> bool;
}
