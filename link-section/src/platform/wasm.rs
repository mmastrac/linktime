//! WASM-specific implementation of the link section.
use core::alloc::Layout;
use core::cell::UnsafeCell;
use core::ptr::{self, NonNull};
use core::sync::atomic::{AtomicU8, Ordering};

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
                        $crate::__support::wasm::LinkSectionInfo::new::<$generic_ty>(__LINK_SECTION_NAME)
                    );
            );

            unsafe { $crate::__support::Bounds::new(&raw const __LINK_SECTION_INFO) }
        }
    }
}

pub use crate::__get_section_wasm as get_section;
use crate::MovableBackref;

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
    (movable, value=$value:expr, slot=$slot:expr, section=$section:tt) => {
        // Register a counting item.
        $crate::__add_section_link_attribute!(
            item data section $section
            #[link_section = __]
            static __LINK_SECTION_COUNTING_ITEM: u8 = 0;
        );

        $crate::__add_section_link_attribute!(
            item data bounds $section
            #[link_name = __]
            extern "C" {
                static __LINK_SECTION_INFO: $crate::__support::wasm::LinkSectionInfoLock<$crate::__support::wasm::LinkSectionMovableInfo>;
            }
        );

        #[link_section = ".init_array.0"]
        #[used] // TODO: used(linker) with linktime_used_linker feature
        #[allow(non_snake_case)]
        static __LINK_SECTION_ITEM_FN_REF: extern "C" fn() = {
            extern "C" fn __LINK_SECTION_ITEM_FN() {
                static DISARMED: ::core::sync::atomic::AtomicBool = ::core::sync::atomic::AtomicBool::new(false);
                if DISARMED.swap(true, ::core::sync::atomic::Ordering::Relaxed) {
                    return;
                }
                unsafe {
                    let ptr = $crate::__support::wasm::register_wasm_link_section_movable_item::<_>(
                        &raw const __LINK_SECTION_INFO,
                        $slot,
                    );
                    ::core::ptr::write(ptr as *mut _, $value);
                }
            }
            __LINK_SECTION_ITEM_FN
        };
    };
    ($section_type:ident, value=$value:expr, $(ref=$ident:ident,)? section=$section:tt) => {
        // Register a counting item
        $crate::__add_section_link_attribute!(
            item data section $section
            #[link_section = __]
            static __LINK_SECTION_COUNTING_ITEM: u8 = 0;
        );

        $crate::__add_section_link_attribute!(
            item data bounds $section
            #[link_name = __]
            extern "C" {
                static __LINK_SECTION_INFO: $crate::__support::wasm::LinkSectionInfoLock<$crate::__support::wasm::LinkSectionInfo>;
            }
        );

        #[link_section = ".init_array.0"]
        #[used] // TODO: used(linker) with linktime_used_linker feature
        #[allow(non_snake_case)]
        static __LINK_SECTION_ITEM_FN_REF: extern "C" fn() = {
            extern "C" fn __LINK_SECTION_ITEM_FN() {
                static DISARMED: ::core::sync::atomic::AtomicBool = ::core::sync::atomic::AtomicBool::new(false);
                if DISARMED.swap(true, ::core::sync::atomic::Ordering::Relaxed) {
                    return;
                }
                unsafe {
                    let ptr = $crate::__support::wasm::register_wasm_link_section_item(&raw const __LINK_SECTION_INFO);
                    ::core::ptr::write(ptr as *mut _, $value);
                    $(
                        $ident.set(ptr);
                    )?
                }
            }
            __LINK_SECTION_ITEM_FN
        };
    }
}

#[cfg(target_family = "wasm")]
#[allow(missing_unsafe_on_extern)] // MSRV
#[cfg_attr(target_os = "unknown", link(wasm_import_module = "env"))]
#[cfg_attr(target_env = "p1", link(wasm_import_module = "env"))]
extern "C" {
    /// Read custom section with name/name_length as a UTF8 string
    pub(crate) fn read_custom_section(
        name: *const u8,
        name_length: usize,
        target_address: *mut u8,
        target_address_length: usize,
    ) -> usize;
}

#[cfg(not(target_family = "wasm"))]
unsafe fn read_custom_section(
    _name: *const u8,
    _name_length: usize,
    _target_address: *mut u8,
    _target_address_length: usize,
) -> usize {
    unreachable!("placeholder for non-WASM platforms")
}

#[repr(u8)]
enum LinkSectionState {
    Uninitialized = 0,
    Initializing = 1,
    Initialized = 2,
}

enum LockState {
    /// The underlying data is not yet initialized.
    Uninitialized = 0,
    /// The underlying data is unlocked. We expect this to be the most common
    /// case.
    Unlocked = 1,
    /// The underlying data is locked.
    Locked = 2,
}

/// The link section. It is expected that the first access through to the final
/// initialization will be single-threaded, but we protect via atomics to ensure
/// safety. Concurrent access during initialization will likely result in a
/// panic (rather than undefined behavior).
///
/// Note that we cannot predict when the first access will be.
#[derive(Clone, Copy)]
pub struct LinkSection<I>(NonNull<LinkSectionInfoLock<I>>);

impl<I: LinkSectionInfoInit> LinkSection<I> {
    /// Get a handle to the lock.
    ///
    /// # Safety
    ///
    /// `info_ptr` must be non-null, properly aligned, and point to the
    /// macro-generated `LinkSectionInfoLock<I>` static for the section's entire
    /// lifetime. The section must only be initialized and accessed from a single
    /// thread during pre-`main` registration and ctor setup (concurrent access
    /// during initialization may panic).
    pub const unsafe fn new(info_ptr: *const LinkSectionInfoLock<I>) -> Self {
        Self(unsafe { NonNull::new_unchecked(info_ptr as *mut _) })
    }

    /// Lock the link section and return a guard.
    #[inline(always)]
    pub fn lock<'a>(&'a self) -> LinkSectionLockGuard<'a, I> {
        let lock_state = unsafe { self.lock_ref() };
        if let Err(old) = lock_state.compare_exchange(
            LockState::Unlocked as _,
            LockState::Locked as _,
            Ordering::Acquire,
            Ordering::Acquire,
        ) {
            self.maybe_lock_uninit(old)
        } else {
            LinkSectionLockGuard(lock_state, unsafe { self.as_mut() })
        }
    }

    #[cold]
    #[inline(never)]
    fn maybe_lock_uninit<'a>(&'a self, old: u8) -> LinkSectionLockGuard<'a, I> {
        let lock_state = unsafe { self.lock_ref() };
        if old == LockState::Uninitialized as u8 {
            if lock_state
                .compare_exchange(
                    LockState::Uninitialized as _,
                    LockState::Locked as _,
                    Ordering::Acquire,
                    Ordering::Acquire,
                )
                .is_err()
            {
                panic!("Link section already being initialized");
            }
            let info = unsafe { self.as_mut() };
            info.initialize();
            LinkSectionLockGuard(lock_state, info)
        } else {
            panic!("Link section already locked");
        }
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

/// The current state of the link section.
#[repr(C)]
pub struct LinkSectionInfoLock<I> {
    lock: AtomicU8,
    info: UnsafeCell<I>,
}

// SAFETY:

// Mutation of `LinkSectionInfo` is guarded by `LinkSection::lock`, which
// synchronize via `AtomicU8`.
unsafe impl<I> Sync for LinkSectionInfoLock<I> {}

/// Initialization behavior for WASM link-section metadata records.
pub trait LinkSectionInfoInit {
    /// Initialize the backing storage and return the number of registered
    /// items expected in the section.
    fn initialize(&mut self) -> usize;
}

/// A record describing the WASM link section.
#[repr(C)]
pub struct LinkSectionInfo {
    state: u8,
    name_length: u16,
    name: *const u8,
    start: *const (),
    end: *const (),
    current: *const (),
    size_of: usize,
    align_of: usize,
}

impl LinkSectionInfo {
    /// Create metadata for a WASM link section storing `T`.
    pub const fn new<T: 'static>(name: &'static str) -> Self {
        Self {
            state: LinkSectionState::Uninitialized as _,
            name_length: name.len() as _,
            name: name.as_ptr(),
            start: ptr::null_mut(),
            end: ptr::null_mut(),
            current: ptr::null_mut(),
            size_of: ::core::mem::size_of::<T>(),
            align_of: ::core::mem::align_of::<T>(),
        }
    }
}

/// A record describing a movable WASM link section and its backref storage.
#[repr(C)]
pub struct LinkSectionMovableInfo {
    base: LinkSectionInfo,
    backrefs_start: *const (),
    backrefs_current: *const (),
    backrefs_end: *const (),
}

const BACKREF_SIZE_OF: usize = ::core::mem::size_of::<crate::MovableBackref<()>>();
const BACKREF_ALIGN_OF: usize = ::core::mem::align_of::<crate::MovableBackref<()>>();

impl LinkSectionMovableInfo {
    /// Create metadata for a movable WASM link section storing `T`.
    pub const fn new<T: 'static>(name: &'static str) -> Self {
        Self {
            base: LinkSectionInfo::new::<T>(name),
            backrefs_start: ptr::null_mut(),
            backrefs_current: ptr::null_mut(),
            backrefs_end: ptr::null_mut(),
        }
    }
}

impl<I> LinkSectionInfoLock<I> {
    /// Create a new link section raw info.
    pub const fn new(info: I) -> Self {
        Self {
            lock: AtomicU8::new(LockState::Uninitialized as _),
            info: UnsafeCell::new(info),
        }
    }
}

impl LinkSectionInfoInit for LinkSectionInfo {
    /// Initialize the link section.
    fn initialize(&mut self) -> usize {
        let size =
            unsafe { read_custom_section(self.name, self.name_length as _, ptr::null_mut(), 0) };

        // We can jump directly to initialized if the section is empty
        if size == 0 {
            // Avoid leaving null pointers behind: `byte_offset_from` and
            // slice creation may be called even for empty sections.
            let dangling = NonNull::<u8>::dangling().as_ptr() as *const ();
            self.start = dangling;
            self.end = dangling;
            self.current = dangling;
            self.state = LinkSectionState::Initialized as _;
            return 0;
        }

        let layout_bytes = size
            .checked_mul(self.size_of)
            .unwrap_or_else(|| panic!("Link section size overflow"));
        unsafe {
            // We got these from a type, so they are always valid
            let ptr =
                allocate(Layout::from_size_align(layout_bytes, self.align_of).unwrap_unchecked());
            if ptr.is_null() {
                panic!("Link section allocation failed");
            }
            self.start = ptr as *const ();
            self.current = ptr as *const ();
            self.end = (ptr as *mut u8).add(layout_bytes) as *const ();
        }
        self.state = LinkSectionState::Initializing as _;

        size
    }
}

impl LinkSectionInfoInit for LinkSectionMovableInfo {
    fn initialize(&mut self) -> usize {
        let size = self.base.initialize();
        if size == 0 {
            let dangling = NonNull::<u8>::dangling().as_ptr() as *const ();
            self.backrefs_start = dangling;
            self.backrefs_current = dangling;
            self.backrefs_end = dangling;
            return 0;
        }

        let layout_bytes = size
            .checked_mul(BACKREF_SIZE_OF)
            .unwrap_or_else(|| panic!("Link section backref size overflow"));
        unsafe {
            // We got these from a type, so they are always valid.
            let ptr = allocate(
                Layout::from_size_align(layout_bytes, BACKREF_ALIGN_OF).unwrap_unchecked(),
            );
            if ptr.is_null() {
                panic!("Link section backref allocation failed");
            }
            self.backrefs_start = ptr as *const ();
            self.backrefs_current = ptr as *const ();
            self.backrefs_end = (ptr as *mut u8).add(layout_bytes) as *const ();
        }

        size
    }
}

/// Register a link section item.
///
/// # Safety
///
/// For macro-generated use only. `info_ptr` must be the matching
/// `LinkSectionInfoLock` static. Must run during single-threaded pre-`main`
/// registration before the section is marked initialized.
pub unsafe fn register_wasm_link_section_item<T>(
    info_ptr: *const LinkSectionInfoLock<LinkSectionInfo>,
) -> *mut T {
    let link_section = unsafe { LinkSection::new(info_ptr) };
    let mut info = link_section.lock();

    unsafe {
        if info.state == LinkSectionState::Initialized as u8 {
            panic!("Link section already initialized");
        }

        let slot = info.current;
        let next = slot.cast::<u8>().add(info.size_of) as *const ();
        if next > info.end {
            panic!("Link section overflow: too many registered items");
        }

        info.current = next;
        if next == info.end {
            info.state = LinkSectionState::Initialized as u8;
        }
        slot as *mut T
    }
}

/// Register a movable link section item and its associated backref slot.
///
/// # Safety
///
/// For macro-generated use only. `info_ptr` and `backref_slot` must be the
/// macro-generated statics for this submission. Must run during single-threaded
/// pre-`main` registration before the section is marked initialized.
pub unsafe fn register_wasm_link_section_movable_item<T: 'static>(
    info_ptr: *const LinkSectionInfoLock<LinkSectionMovableInfo>,
    backref_slot: *const UnsafeCell<*const T>,
) -> *mut T {
    let link_section = unsafe { LinkSection::new(info_ptr) };
    let mut info = link_section.lock();

    unsafe {
        if info.base.state == LinkSectionState::Initialized as u8 {
            panic!("Link section already initialized");
        }

        let slot = info.base.current;
        let next = slot.cast::<u8>().add(info.base.size_of) as *const ();
        if next > info.base.end {
            panic!("Link section overflow: too many registered items");
        }
        info.base.current = next;

        let backref = info.backrefs_current as *mut MovableBackref<T>;
        let backref_next = backref.cast::<u8>().add(BACKREF_SIZE_OF) as *const ();
        if backref_next > info.backrefs_end {
            panic!("Link section backref overflow: too many registered items");
        }
        info.backrefs_current = backref_next;

        if next == info.base.end {
            info.base.state = LinkSectionState::Initialized as u8;
        }

        // Create a new backref record and set the existing slot to the new item.
        ptr::write(backref, MovableBackref::new(backref_slot));
        ptr::write(UnsafeCell::raw_get(backref_slot), slot.cast());

        slot as *mut T
    }
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

/// On WASM, we use an atomic pointer to the start and end of the
/// section. The host environment is responsible for registering the
/// section with the runtime.
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

    /// Get the start pointer of the link section.
    pub fn start_ptr(&self) -> *const () {
        let lock = self.0.lock();
        if lock.state != LinkSectionState::Initialized as u8 {
            panic!("Link section not initialized: possible ctor ordering issue");
        }
        lock.start
    }

    /// Get the end pointer of the link section.
    pub fn end_ptr(&self) -> *const () {
        let lock = self.0.lock();
        if lock.state != LinkSectionState::Initialized as u8 {
            panic!("Link section not initialized: possible ctor ordering issue");
        }
        lock.end
    }

    /// This is intentionally safe to call before the section is fully
    /// initialized.
    pub fn byte_len(&self) -> usize {
        let lock = self.0.lock();
        unsafe { (lock.end.cast::<u8>()).offset_from(lock.start.cast::<u8>()) as usize }
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

    /// Get the start pointer of the movable item section.
    pub fn start_ptr(&self) -> *const () {
        let lock = self.0.lock();
        if lock.base.state != LinkSectionState::Initialized as u8 {
            panic!("Link section not initialized: possible ctor ordering issue");
        }
        lock.base.start
    }

    /// Get the end pointer of the movable item section.
    pub fn end_ptr(&self) -> *const () {
        let lock = self.0.lock();
        if lock.base.state != LinkSectionState::Initialized as u8 {
            panic!("Link section not initialized: possible ctor ordering issue");
        }
        lock.base.end
    }

    /// This is intentionally safe to call before the section is fully
    /// initialized.
    pub fn byte_len(&self) -> usize {
        let lock = self.0.lock();
        unsafe { (lock.base.end.cast::<u8>()).offset_from(lock.base.start.cast::<u8>()) as usize }
    }

    /// Get the start pointer of the movable backref section.
    pub fn backrefs_start_ptr(&self) -> *const () {
        let lock = self.0.lock();
        if lock.base.state != LinkSectionState::Initialized as u8 {
            panic!("Link section not initialized: possible ctor ordering issue");
        }
        lock.backrefs_start
    }

    /// Get the end pointer of the movable backref section.
    pub fn backrefs_end_ptr(&self) -> *const () {
        let lock = self.0.lock();
        if lock.base.state != LinkSectionState::Initialized as u8 {
            panic!("Link section not initialized: possible ctor ordering issue");
        }
        lock.backrefs_end
    }

    /// This is intentionally safe to call before the section is fully
    /// initialized.
    pub fn backrefs_byte_len(&self) -> usize {
        let lock = self.0.lock();
        unsafe {
            (lock.backrefs_end.cast::<u8>()).offset_from(lock.backrefs_start.cast::<u8>()) as usize
        }
    }
}
