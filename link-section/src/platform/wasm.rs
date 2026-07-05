//! WASM-specific implementation of the link section.
//!
//! WASM has no linker-gathered sections whose bounds are available at runtime.
//! Each submission emits a single intrusive [`LinkCell`] static: the value `T`
//! at offset 0 with list metadata trailing it, plus an `.init_array.0`
//! constructor that threads the cell onto a singly-linked list. The cells are
//! interior-mutable and address-significant, so LLVM cannot merge them even
//! under fat LTO with identical values.
//!
//! Sections that hand out `Ref`/`MovableRef` handles carry a fix-up slot after
//! the list pointer ([`LinkMetaSlot`]). Plain `typed`/`mutable` sections do not
//! ([`LinkMeta`]).
//!
//! The list is flattened into contiguous storage either by the "backstop" ctor
//! that runs in `.init_array.1`, or lazily on the first bounds/ref access.

use core::alloc::Layout;
use core::cell::UnsafeCell;
use core::ptr::{self, NonNull};
use core::sync::atomic::{AtomicU8, Ordering};

use super::{SectionRange, SyncUnsafeCell};

#[doc(hidden)]
#[macro_export]
macro_rules! __get_section_wasm {
    (movable, name=$name:tt, type=$generic_ty:ty) => {
        {
            static __LINK_SECTION_NAME: &'static str = $crate::__support::section_name!(
                string item data bare $name
            );
            $crate::__support::add_section_link_attribute!(
                item data bounds $name
                #[export_name = __]
                #[used]
                static __LINK_SECTION_INFO: $crate::__support::wasm::LinkSectionInfoLock<$crate::__support::wasm::LinkSectionMovableInfo> =
                    $crate::__support::wasm::LinkSectionInfoLock::new(
                        $crate::__support::wasm::LinkSectionMovableInfo::new::<$generic_ty>(__LINK_SECTION_NAME)
                    );
            );

            // Eager finalization at `.init_array.1` (submissions are `.init_array.0`).
            #[link_section = ".init_array.1"]
            #[used]
            #[allow(non_snake_case)]
            static __LINK_SECTION_FLATTEN_FN_REF: extern "C" fn() = {
                extern "C" fn __LINK_SECTION_FLATTEN_FN() {
                    unsafe {
                        $crate::__support::wasm::flatten_wasm_link_section_movable(
                            &raw const __LINK_SECTION_INFO,
                        );
                    }
                }
                __LINK_SECTION_FLATTEN_FN
            };

            unsafe { $crate::__support::MovableBounds::new(&raw const __LINK_SECTION_INFO) }
        }
    };
    ($section_type:ident, name=$name:tt, type=$generic_ty:ty) => {
        {
            static __LINK_SECTION_NAME: &'static str = $crate::__support::section_name!(
                string item data bare $name
            );
            $crate::__support::add_section_link_attribute!(
                item data bounds $name
                #[export_name = __]
                #[used]
                static __LINK_SECTION_INFO: $crate::__support::wasm::LinkSectionInfoLock<$crate::__support::wasm::LinkSectionInfo> =
                    $crate::__support::wasm::LinkSectionInfoLock::new(
                        // Only `reference` sections need per-item fix-up slots.
                        $crate::__support::wasm::LinkSectionInfo::new::<$generic_ty>(
                            __LINK_SECTION_NAME,
                            $crate::__support::wasm::section_has_slot!($section_type),
                        )
                    );
            );

            // Eager finalization at `.init_array.1` (see the movable arm).
            #[link_section = ".init_array.1"]
            #[used]
            #[allow(non_snake_case)]
            static __LINK_SECTION_FLATTEN_FN_REF: extern "C" fn() = {
                extern "C" fn __LINK_SECTION_FLATTEN_FN() {
                    unsafe {
                        $crate::__support::wasm::flatten_wasm_link_section(
                            &raw const __LINK_SECTION_INFO,
                        );
                    }
                }
                __LINK_SECTION_FLATTEN_FN
            };

            unsafe { $crate::__support::Bounds::new(&raw const __LINK_SECTION_INFO) }
        }
    }
}

pub use crate::__get_section_wasm as get_section;

/// Whether a section kind hands out reference handles and so needs a per-item
/// fix-up slot. Only `reference` sections do. Expands to a `const bool`.
#[doc(hidden)]
#[macro_export]
macro_rules! __section_has_slot {
    (reference) => {
        true
    };
    ($other:ident) => {
        false
    };
}
pub use crate::__section_has_slot as section_has_slot;

crate::__def_section_name! {
    __section_name_wasm,
    {
        data bare =>    (".data" ".link_section.") __ ();
        data section => (".data" ".link_section.") __ ();
        code bare =>    (".text" ".link_section.") __ ();
        code section => (".text" ".link_section.") __ ();
        data bounds =>  (".data" ".link_section.") __ (".bounds");
    }
    AUXILIARY = ".";
    REFS = ".r.";
    MAX_LENGTH = 16;
    HASH_LENGTH = 6;
    VALID_SECTION_CHARS = "_ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
}

#[cfg(not(target_family = "wasm"))]
#[doc(hidden)]
#[macro_export]
#[allow(unknown_lints, edition_2024_expr_fragment_specifier)]
macro_rules! __register_wasm_item {
    ($($args:tt)*) => {};
}

#[cfg(target_family = "wasm")]
#[doc(hidden)]
#[macro_export]
#[allow(unknown_lints, edition_2024_expr_fragment_specifier)]
macro_rules! __register_wasm_item {
    // Movable section: carry the `MovableRef` slot for flatten to fix up.
    (movable, type=$ty:ty, value=$value:expr, slot=$slot:expr, section=$section:tt) => {
        $crate::__register_wasm_item!(@emit
            info_ty = ($crate::__support::wasm::LinkSectionMovableInfo),
            register = register_wasm_link_section_movable_item,
            ty = ($ty),
            meta = ($crate::__support::wasm::LinkMetaSlot),
            new = ($value, $slot as *const ()),
            section = $section
        );
    };
    // Reference section: carry the `Ref` slot for flatten to fix up.
    ($section_type:ident, type=$ty:ty, value=$value:expr, ref=$ident:ident, section=$section:tt) => {
        $crate::__register_wasm_item!(@emit
            info_ty = ($crate::__support::wasm::LinkSectionInfo),
            register = register_wasm_link_section_item,
            ty = ($ty),
            meta = ($crate::__support::wasm::LinkMetaSlot),
            new = ($value, &$ident as *const _ as *const ()),
            section = $section
        );
    };
    // Typed / mutable const items: value is copied into the section, no slot.
    ($section_type:ident, type=$ty:ty, value=$value:expr, section=$section:tt) => {
        $crate::__register_wasm_item!(@emit
            info_ty = ($crate::__support::wasm::LinkSectionInfo),
            register = register_wasm_link_section_item,
            ty = ($ty),
            meta = ($crate::__support::wasm::LinkMeta),
            new = ($value),
            section = $section
        );
    };
    (@emit
        info_ty = ($info_ty:ty),
        register = $register:ident,
        ty = ($ty:ty),
        meta = ($meta:ty),
        new = ($($new_args:expr),*),
        section = $section:tt
    ) => {
        #[used]
        static __LINK_SECTION_CELL: $crate::__support::wasm::LinkCell<$ty, $meta> =
            <$crate::__support::wasm::LinkCell<$ty, $meta>>::new($($new_args),*);

        $crate::__add_section_link_attribute!(
            item data bounds $section
            #[link_name = __]
            extern "C" {
                static __LINK_SECTION_INFO: $crate::__support::wasm::LinkSectionInfoLock<$info_ty>;
            }
        );

        #[link_section = ".init_array.0"]
        #[used] // TODO: used(linker) with linktime_used_linker feature
        #[allow(non_snake_case)]
        static __LINK_SECTION_ITEM_FN_REF: extern "C" fn() = {
            extern "C" fn __LINK_SECTION_ITEM_FN() {
                static DISARMED: ::core::sync::atomic::AtomicBool =
                    ::core::sync::atomic::AtomicBool::new(false);
                if DISARMED.swap(true, ::core::sync::atomic::Ordering::Relaxed) {
                    return;
                }
                unsafe {
                    $crate::__support::wasm::$register(
                        &raw const __LINK_SECTION_INFO,
                        __LINK_SECTION_CELL.as_cell_ptr(),
                    );
                }
            }
            __LINK_SECTION_ITEM_FN
        };
    };
}

/// The lifecycle state of a section's storage.
#[repr(u8)]
enum LinkSectionState {
    /// Items are still being submitted. No contiguous storage exists yet.
    Gathering = 0,
    /// The list has been materialised into contiguous storage.
    Flattened = 2,
}

/// The lock state of a section's info record.
enum LockState {
    /// The info record is unlocked (the common case).
    Unlocked = 1,
    /// The info record is locked.
    Locked = 2,
}

/// Intrusive list link for a plain (`typed`/`mutable`) cell: just the pointer to
/// the next cell in the list. Trails the value in a [`LinkCell`].
#[repr(C)]
pub struct LinkMeta {
    /// Start of the next cell in the list (written at registration). Null for the tail.
    next: UnsafeCell<*mut u8>,
}

/// Intrusive list link for a reference (`reference`/`movable`) cell: the next
/// pointer, plus a type-erased `*const UnsafeCell<*const T>` fix-up slot that
/// flatten points at the item's final location. `next` stays first so it lives
/// at the same offset as [`LinkMeta`]'s `next`.
#[repr(C)]
pub struct LinkMetaSlot {
    next: UnsafeCell<*mut u8>,
    slot: *const (),
}

/// Byte offset of the fix-up slot within [`LinkMetaSlot`] (it trails `next`).
const SLOT_OFFSET: usize = core::mem::size_of::<*mut u8>();

/// Offset of the trailing link metadata within a cell, given the value size.
///
/// The value is the leading field and the link is pointer-aligned, so this is
/// `size_of::<T>()` rounded up to the pointer alignment. It depends only on the
/// value size, not on `T` or the link type, so flatten recovers it at runtime
/// from the stored `size_of`.
const fn meta_offset(size_of: usize) -> usize {
    let align = core::mem::align_of::<*mut u8>();
    (size_of + align - 1) & !(align - 1)
}

/// Intrusive list node emitted once per submitted item. The value `T` is the
/// leading field (offset 0) so flatten copies `size_of::<T>()` bytes straight
/// from the cell's address, and the link metadata `M` trails it. The cell is
/// interior-mutable and address-significant, so LLVM can't fold identical cells
/// even under fat LTO. That is what keeps the item count LTO-safe.
#[repr(C)]
pub struct LinkCell<T: 'static, M> {
    value: UnsafeCell<T>,
    meta: M,
}

// SAFETY: `value` is set at construction and read only under the section lock
// at flatten. The trailing link is written only under that lock.
unsafe impl<T: 'static, M> Sync for LinkCell<T, M> {}

impl<T: 'static> LinkCell<T, LinkMeta> {
    /// Create a plain cell embedding `value` (no fix-up slot).
    pub const fn new(value: T) -> Self {
        Self {
            value: UnsafeCell::new(value),
            meta: LinkMeta {
                next: UnsafeCell::new(ptr::null_mut()),
            },
        }
    }
}

impl<T: 'static> LinkCell<T, LinkMetaSlot> {
    /// Create a cell embedding `value` and a stable `slot` for flatten to fix up
    /// (`Ref`/`MovableRef` sections).
    pub const fn new(value: T, slot: *const ()) -> Self {
        Self {
            value: UnsafeCell::new(value),
            meta: LinkMetaSlot {
                next: UnsafeCell::new(ptr::null_mut()),
                slot,
            },
        }
    }
}

impl<T: 'static, M> LinkCell<T, M> {
    /// Pointer to the start of this cell (the value). Stored in the section's
    /// list and in each cell's `meta.next`.
    pub const fn as_cell_ptr(&self) -> *mut u8 {
        self as *const LinkCell<T, M> as *mut u8
    }
}

/// The link section handle. Registration and the first (flattening) read are
/// single-threaded. An atomic lock makes concurrent misuse panic instead of UB.
#[derive(Clone, Copy)]
pub struct LinkSection<I>(NonNull<LinkSectionInfoLock<I>>);

impl<I> LinkSection<I> {
    /// Get a handle to the section info.
    ///
    /// # Safety
    ///
    /// `info_ptr` must point to the macro-generated `LinkSectionInfoLock<I>`
    /// static for the section's entire lifetime. Registration and first access
    /// must be single-threaded, pre-`main`.
    pub const unsafe fn new(info_ptr: *const LinkSectionInfoLock<I>) -> Self {
        Self(unsafe { NonNull::new_unchecked(info_ptr as *mut _) })
    }

    /// Lock the section info and return a guard.
    #[inline]
    pub fn lock(&self) -> LinkSectionLockGuard<'_, I> {
        let lock_state = unsafe { self.lock_ref() };
        if lock_state
            .compare_exchange(
                LockState::Unlocked as _,
                LockState::Locked as _,
                Ordering::Acquire,
                Ordering::Acquire,
            )
            .is_err()
        {
            panic!("Link section already locked: concurrent access is not supported");
        }
        LinkSectionLockGuard(lock_state, unsafe { self.as_mut() })
    }

    #[inline(always)]
    unsafe fn lock_ref(&self) -> &AtomicU8 {
        // as_ref_unchecked when we bump MSRV
        unsafe {
            ptr::addr_of!((*self.0.as_ptr()).lock)
                .as_ref()
                .unwrap_unchecked()
        }
    }

    #[inline(always)]
    #[allow(clippy::mut_from_ref)]
    unsafe fn as_mut(&self) -> &mut I {
        unsafe {
            let unsafe_cell = ptr::addr_of!((*self.0.as_ptr()).info);
            // as_mut_unchecked when we bump MSRV
            UnsafeCell::raw_get(unsafe_cell).as_mut().unwrap_unchecked()
        }
    }
}

/// Lightweight lock guard for the link section.
pub struct LinkSectionLockGuard<'a, I>(&'a AtomicU8, &'a mut I);
impl<'a, I> core::ops::Deref for LinkSectionLockGuard<'a, I> {
    type Target = I;
    fn deref(&self) -> &Self::Target {
        self.1
    }
}
impl<'a, I> core::ops::DerefMut for LinkSectionLockGuard<'a, I> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.1
    }
}
impl<'a, I> Drop for LinkSectionLockGuard<'a, I> {
    fn drop(&mut self) {
        self.0.store(LockState::Unlocked as _, Ordering::Release);
    }
}

/// Lock + info record for a section.
#[repr(C)]
pub struct LinkSectionInfoLock<I> {
    lock: AtomicU8,
    info: UnsafeCell<I>,
}

// SAFETY: info mutation is guarded by `LinkSection::lock` (via `AtomicU8`).
unsafe impl<I> Sync for LinkSectionInfoLock<I> {}

impl<I> LinkSectionInfoLock<I> {
    /// Create a new link section lock + info.
    pub const fn new(info: I) -> Self {
        Self {
            lock: AtomicU8::new(LockState::Unlocked as _),
            info: UnsafeCell::new(info),
        }
    }
}

/// A WASM link section: the intrusive submission list and, once flattened, the
/// contiguous storage bounds.
#[repr(C)]
pub struct LinkSectionInfo {
    state: u8,
    /// Whether cells carry a [`LinkMetaSlot`] fix-up slot (`reference`/`movable`
    /// sections) rather than a bare [`LinkMeta`] (`typed`/`mutable` sections).
    has_slot: bool,
    head: *mut u8,
    count: usize,
    start: *const (),
    end: *const (),
    size_of: usize,
    align_of: usize,
    name: &'static str,
}

impl LinkSectionInfo {
    /// Create metadata for a WASM link section storing `T`. `has_slot` selects
    /// [`LinkMetaSlot`] cells (reference sections) over [`LinkMeta`] cells.
    pub const fn new<T: 'static>(name: &'static str, has_slot: bool) -> Self {
        Self {
            state: LinkSectionState::Gathering as _,
            has_slot,
            head: ptr::null_mut(),
            count: 0,
            start: ptr::null(),
            end: ptr::null(),
            size_of: ::core::mem::size_of::<T>(),
            align_of: ::core::mem::align_of::<T>(),
            name,
        }
    }

    /// Thread a cell onto the front of the list. Panics if already flattened.
    ///
    /// # Safety
    ///
    /// `cell` must be the start of a `LinkCell` whose value type matches this
    /// section's `size_of`.
    unsafe fn push(&mut self, cell: *mut u8) {
        if self.state == LinkSectionState::Flattened as u8 {
            panic!("Link section already initialized");
        }
        unsafe {
            let next = cell.add(meta_offset(self.size_of)) as *mut *mut u8;
            *next = self.head;
        }
        self.head = cell;
        self.count += 1;
    }

    /// Allocate contiguous storage for `count` values and copy the list into it
    /// back-to-front so contiguous order matches submission order. Returns the
    /// allocation, or fixes up empty sections and returns `None`.
    ///
    /// For each cell, if `has_slot`, points its fix-up slot at the item's final
    /// location and invokes `on_slot(index, slot)` (used by movable sections to
    /// record backrefs).
    fn materialise(&mut self, mut on_slot: impl FnMut(usize, *const ())) -> Option<*mut u8> {
        let count = self.count;
        if count == 0 {
            let dangling = NonNull::<u8>::dangling().as_ptr() as *const ();
            self.start = dangling;
            self.end = dangling;
            return None;
        }

        let size_of = self.size_of;
        let bytes = count
            .checked_mul(size_of)
            .unwrap_or_else(|| panic!("Link section size overflow"));
        let arr =
            unsafe { allocate(Layout::from_size_align(bytes, self.align_of).unwrap()) as *mut u8 };
        if arr.is_null() {
            panic!("Link section allocation failed");
        }

        let meta_offset = meta_offset(size_of);
        let mut cell = self.head;
        let mut i = count;
        while !cell.is_null() {
            i -= 1;
            unsafe {
                let dst = arr.add(i * size_of);
                // Value is the leading field, at offset 0.
                ptr::copy_nonoverlapping(cell, dst, size_of);

                let meta = cell.add(meta_offset);
                if self.has_slot {
                    let slot = *(meta.add(SLOT_OFFSET) as *const *const ());
                    if !slot.is_null() {
                        ptr::write(
                            UnsafeCell::raw_get(slot as *const UnsafeCell<*const ()>),
                            dst as *const (),
                        );
                    }
                    on_slot(i, slot);
                }
                // `next` is the leading field of the link metadata.
                cell = *(meta as *const *mut u8);
            }
        }

        self.start = arr as *const ();
        self.end = unsafe { arr.add(bytes) } as *const ();
        Some(arr)
    }

    /// Materialise the intrusive list into one contiguous allocation. Idempotent.
    fn flatten(&mut self) {
        if self.state == LinkSectionState::Flattened as u8 {
            return;
        }
        self.materialise(|_, _| {});
        self.state = LinkSectionState::Flattened as u8;
    }

    /// The flattened item range. Only valid once [`flatten`](Self::flatten) has run.
    fn range(&self) -> SectionRange {
        SectionRange::new(self.start, self.end)
    }
}

/// A movable WASM link section and its backref storage.
#[repr(C)]
pub struct LinkSectionMovableInfo {
    base: LinkSectionInfo,
    backrefs_start: *const (),
    backrefs_end: *const (),
}

const BACKREF_SIZE_OF: usize = ::core::mem::size_of::<crate::MovableBackref<()>>();
const BACKREF_ALIGN_OF: usize = ::core::mem::align_of::<crate::MovableBackref<()>>();

impl LinkSectionMovableInfo {
    /// Create metadata for a movable WASM link section storing `T`.
    pub const fn new<T: 'static>(name: &'static str) -> Self {
        Self {
            // Movable sections hand out `MovableRef`s, so cells carry a slot.
            base: LinkSectionInfo::new::<T>(name, true),
            backrefs_start: ptr::null(),
            backrefs_end: ptr::null(),
        }
    }

    /// Materialise the values and backref arrays. Idempotent.
    ///
    /// For each item: copies the value into the value array, points the item's
    /// `MovableRef` slot at its final location, and writes a backref record
    /// (a pointer to that slot) into the backref array.
    fn flatten(&mut self) {
        if self.base.state == LinkSectionState::Flattened as u8 {
            return;
        }

        let count = self.base.count;
        let (backrefs, backref_bytes) = if count == 0 {
            let dangling = NonNull::<u8>::dangling().as_ptr() as *const ();
            (dangling as *mut u8, 0)
        } else {
            let backref_bytes = count
                .checked_mul(BACKREF_SIZE_OF)
                .unwrap_or_else(|| panic!("Link section backref size overflow"));
            let backrefs = unsafe {
                allocate(Layout::from_size_align(backref_bytes, BACKREF_ALIGN_OF).unwrap())
                    as *mut u8
            };
            if backrefs.is_null() {
                panic!("Link section backref allocation failed");
            }
            (backrefs, backref_bytes)
        };

        // Copy values and record a backref at each item's fixed-up slot.
        self.base.materialise(|i, slot| unsafe {
            // `MovableBackref<T>` is a single pointer (`#[repr(C)] { slot }`).
            let br = backrefs.add(i * BACKREF_SIZE_OF) as *mut *const ();
            ptr::write(br, slot);
        });

        self.backrefs_start = backrefs as *const ();
        self.backrefs_end = unsafe { backrefs.add(backref_bytes) } as *const ();
        self.base.state = LinkSectionState::Flattened as u8;
    }

    /// The flattened backref range. Only valid once [`flatten`](Self::flatten) has run.
    fn backrefs_range(&self) -> SectionRange {
        SectionRange::new(self.backrefs_start, self.backrefs_end)
    }
}

/// Thread a cell onto the section's intrusive list.
///
/// # Safety
///
/// For macro-generated use only. `info_ptr` must be the matching
/// `LinkSectionInfoLock` static and `cell` the start of the item's `LinkCell`
/// (see [`LinkCell::as_cell_ptr`]). Must run single-threaded, pre-`main`, before
/// the section is flattened.
pub unsafe fn register_wasm_link_section_item(
    info_ptr: *const LinkSectionInfoLock<LinkSectionInfo>,
    cell: *mut u8,
) {
    let link_section = unsafe { LinkSection::new(info_ptr) };
    unsafe { link_section.lock().push(cell) };
}

/// Thread a movable-section cell onto the list.
///
/// # Safety
///
/// As [`register_wasm_link_section_item`], but `cell`'s slot must be the item's
/// `MovableRef` slot.
pub unsafe fn register_wasm_link_section_movable_item(
    info_ptr: *const LinkSectionInfoLock<LinkSectionMovableInfo>,
    cell: *mut u8,
) {
    let link_section = unsafe { LinkSection::new(info_ptr) };
    unsafe { link_section.lock().base.push(cell) };
}

/// Flatten a link section (called eagerly by the `.init_array.1` finalizer, and
/// as an idempotent backstop from `Ref` deref and section reads).
///
/// # Safety
///
/// For macro-generated use only. `info_ptr` must be the section's
/// `LinkSectionInfoLock` static.
pub unsafe fn flatten_wasm_link_section(info_ptr: *const LinkSectionInfoLock<LinkSectionInfo>) {
    let link_section = unsafe { LinkSection::new(info_ptr) };
    link_section.lock().flatten();
}

/// Flatten a movable link section. See [`flatten_wasm_link_section`].
///
/// # Safety
///
/// For macro-generated use only. `info_ptr` must be the movable section's
/// `LinkSectionInfoLock` static.
pub unsafe fn flatten_wasm_link_section_movable(
    info_ptr: *const LinkSectionInfoLock<LinkSectionMovableInfo>,
) {
    let link_section = unsafe { LinkSection::new(info_ptr) };
    link_section.lock().flatten();
}

#[cfg(target_family = "wasm")]
unsafe fn allocate(layout: Layout) -> *mut () {
    use alloc::alloc::alloc;

    alloc(layout) as _
}

#[cfg(not(target_family = "wasm"))]
unsafe fn allocate(_layout: Layout) -> *mut () {
    unreachable!("placeholder for non-WASM platforms")
}

/// Runtime bounds for a WASM link section, flattened lazily on first access.
pub struct Bounds(LinkSection<LinkSectionInfo>);

impl Bounds {
    /// Create a new bounds struct.
    ///
    /// # Safety
    ///
    /// For macro-generated use only. `info_ptr` must be the section's
    /// `LinkSectionInfoLock` static (see [`LinkSection::new`]).
    pub const unsafe fn new(info_ptr: *const LinkSectionInfoLock<LinkSectionInfo>) -> Self {
        unsafe { Self(LinkSection::new(info_ptr)) }
    }

    /// Resolve the section into an immutable [`SectionRange`], flattening it
    /// (idempotently) if some earlier access has not already done so. Locks once.
    pub fn range(&self) -> SectionRange {
        let mut info = self.0.lock();
        info.flatten();
        info.range()
    }
}

/// Runtime bounds for a WASM movable link section and its backref section.
pub struct MovableBounds(LinkSection<LinkSectionMovableInfo>);

impl MovableBounds {
    /// Create a new movable bounds struct.
    ///
    /// # Safety
    ///
    /// For macro-generated use only. `info_ptr` must be the movable section's
    /// `LinkSectionInfoLock` static (see [`LinkSection::new`]).
    pub const unsafe fn new(info_ptr: *const LinkSectionInfoLock<LinkSectionMovableInfo>) -> Self {
        unsafe { Self(LinkSection::new(info_ptr)) }
    }

    /// Resolve the movable item section into an immutable [`SectionRange`],
    /// flattening it (idempotently) if necessary. Locks once.
    pub fn range(&self) -> SectionRange {
        let mut info = self.0.lock();
        info.flatten();
        info.base.range()
    }

    /// Resolve the movable backref section into an immutable [`SectionRange`],
    /// flattening it (idempotently) if necessary. Locks once.
    pub fn backrefs_range(&self) -> SectionRange {
        let mut info = self.0.lock();
        info.flatten();
        info.backrefs_range()
    }
}

/// Platform storage backing a [`crate::Ref`] on WASM: a lazily-populated pointer
/// slot plus the owning section for the flatten backstop. `ptr` is the first
/// field, so the registration slot (`&Ref`) points straight at it.
#[repr(C)]
pub struct RefStorage<T: 'static> {
    ptr: UnsafeCell<*const T>,
    section: *const LinkSectionInfoLock<LinkSectionInfo>,
}

impl<T> RefStorage<T> {
    /// Storage for a `Ref` into `section`, resolved lazily on first deref.
    pub const fn new(section: *const LinkSectionInfoLock<LinkSectionInfo>) -> Self {
        Self {
            ptr: UnsafeCell::new(ptr::null()),
            section,
        }
    }

    /// The item pointer, read without flattening ([`Self::get`] flattens first).
    pub fn as_ptr(&self) -> *const T {
        unsafe { *self.ptr.get() }
    }

    /// Resolve to the item, flattening the section on first touch as an
    /// idempotent backstop for a `#[ctor(priority = 1)]` that runs before the
    /// `.init_array.1` finalizer.
    pub fn get(&self) -> &T {
        unsafe {
            if (*self.ptr.get()).is_null() {
                flatten_wasm_link_section(self.section);
            }
            self.as_ptr().as_ref().expect("Ref not initialized")
        }
    }
}

/// Platform storage backing a [`crate::MovableRef`] on WASM: the stable pointer
/// slot plus the owning section for the flatten backstop. `slot` is the first
/// field, so [`crate::MovableRef::slot_ptr`] is an offset-0 cast.
#[repr(C)]
pub struct MovableRefStorage<T: 'static> {
    slot: SyncUnsafeCell<*const T>,
    section: *const LinkSectionInfoLock<LinkSectionMovableInfo>,
}

impl<T> MovableRefStorage<T> {
    /// Storage for a `MovableRef` into `section`, resolved lazily on first deref.
    pub const fn new(
        ptr: *const T,
        section: *const LinkSectionInfoLock<LinkSectionMovableInfo>,
    ) -> Self {
        Self {
            slot: SyncUnsafeCell::new(ptr),
            section,
        }
    }

    /// The current slot pointer, read without flattening ([`Self::get`] flattens first).
    pub const fn as_ptr(&self) -> *const T {
        unsafe { *self.slot.get() }
    }

    /// Resolve to the item, flattening the section on first touch as an
    /// idempotent backstop. See [`RefStorage::get`].
    pub fn get(&self) -> &T {
        unsafe {
            if (*self.slot.get()).is_null() {
                flatten_wasm_link_section_movable(self.section);
            }
            self.as_ptr().as_ref().expect("MovableRef not initialized")
        }
    }
}
