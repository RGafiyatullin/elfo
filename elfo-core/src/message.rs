use std::{
    alloc, fmt,
    ptr::{self, NonNull},
};

use metrics::Label;
use serde::{Deserialize, Serialize};
use smallbox::smallbox;

use crate::dumping;

pub use self::{any::*, protocol::*, repr::*};

mod any;
mod protocol;
mod repr;

// === Message ===

/// Represents a message that can be sent between actors and across nodes.
///
/// Never implement it by hand, use the `#[message]` macro instead.
pub trait Message:
    fmt::Debug + Clone + Send + Serialize + for<'de> Deserialize<'de> + 'static
{
    #[inline(always)]
    fn name(&self) -> &'static str {
        self._vtable().name
    }

    #[inline(always)]
    fn protocol(&self) -> &'static str {
        self._vtable().protocol
    }

    #[doc(hidden)] // unstable because depends on `metrics`
    #[inline(always)]
    fn labels(&self) -> &'static [Label] {
        &self._vtable().labels
    }

    #[doc(hidden)] // unstable because will be replaced with `DumpingMode`
    #[inline(always)]
    fn dumping_allowed(&self) -> bool {
        self._vtable().dumping_allowed
    }

    #[deprecated(note = "use `AnyMessage::new` instead")]
    #[doc(hidden)]
    #[inline(always)]
    fn upcast(self) -> AnyMessage {
        self._into_any()
    }

    // Private API.

    #[doc(hidden)]
    fn _type_id() -> MessageTypeId;

    #[doc(hidden)]
    fn _vtable(&self) -> &'static MessageVTable;

    #[doc(hidden)]
    #[inline(always)]
    fn _repr_layout(&self) -> alloc::Layout {
        self._vtable().repr_layout
    }

    // NOTE: All methods below MUST be overriden for `AnyMessage`.

    #[doc(hidden)]
    #[inline(always)]
    fn _is_supertype_of(type_id: MessageTypeId) -> bool {
        Self::_type_id() == type_id
    }

    #[doc(hidden)]
    #[inline(always)]
    fn _into_any(self) -> AnyMessage {
        AnyMessage::from_real(self)
    }

    /// # Safety
    ///
    /// The caller must ensure that `any` holds this message type.
    #[doc(hidden)]
    #[inline(always)]
    unsafe fn _from_any(any: AnyMessage) -> Self {
        any.into_real()
    }

    /// # Safety
    ///
    /// The caller must ensure that `any` holds this message type.
    #[doc(hidden)]
    #[inline(always)]
    unsafe fn _from_any_ref(any: &AnyMessage) -> &Self {
        any.as_real_ref()
    }

    #[doc(hidden)]
    #[inline(always)]
    fn _erase(&self) -> dumping::ErasedMessage {
        smallbox!(self.clone())
    }

    /// # Safety
    ///
    /// Behavior is undefined if any of the following conditions are violated:
    /// * `ptr` must be [valid] for reads.
    /// * `ptr` must point to a properly initialized value of type `Self`.
    /// * Data behind `ptr` must not be used after this function is called.
    ///
    /// [valid]: https://doc.rust-lang.org/stable/std/ptr/index.html#safety
    #[doc(hidden)]
    #[inline(always)]
    unsafe fn _read(ptr: NonNull<MessageRepr>) -> Self {
        let data_ref = &ptr.cast::<MessageRepr<Self>>().as_ref().data;
        ptr::read(data_ref)
    }

    /// # Safety
    ///
    /// Behavior is undefined if any of the following conditions are violated:
    /// * `ptr` must be [valid] for writes.
    /// * `ptr` must be properly aligned.
    ///
    /// [valid]: https://doc.rust-lang.org/stable/std/ptr/index.html#safety
    #[doc(hidden)]
    #[inline(always)]
    unsafe fn _write(self, ptr: NonNull<MessageRepr>) {
        let repr = MessageRepr::new(self);
        ptr::write(ptr.cast::<MessageRepr<Self>>().as_ptr(), repr);
    }
}

// === Request ===

/// Represents a request that can be sent between actors and across nodes.
///
/// Never implement it by hand, use the `#[message(ret = ...)]` macro instead.
pub trait Request: Message {
    type Response; // constraints are checked by `Wrapper`

    /// Generated by `#[message(ret = ...)]`.
    /// It allows to use `!Message` such as `Result<T, E>`, `Option<T>` etc.
    #[doc(hidden)]
    type Wrapper: Message + Into<Self::Response> + From<Self::Response>;
}
