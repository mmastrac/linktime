//! WASM-specific implementation of the link section.
use core::alloc::Layout;
use core::cell::UnsafeCell;
use core::ptr::{self, NonNull};
use core::sync::atomic::{AtomicU8, Ordering};

#[doc(hidden)]
#[macro_export]
macro_rules! __get_section_wasm {
    (name=$ident:ident, type=$generic_ty:ty $(, aux=$aux:ident )?) => {
        {
            static __LINK_SECTION_NAME: &'static str = $crate::__support::section_name!(
                raw data bare $ident $($aux)?
            );
            $crate::__support::add_section_link_attribute!(
                data bounds $ident $($aux)?
                #[export_name = __]
                #[used]
                static __LINK_SECTION_INFO: $crate::__support::wasm::LinkSectionRawInfo = $crate::__support::wasm::LinkSectionRawInfo::new::<$generic_ty>(__LINK_SECTION_NAME);
            );

            unsafe { $crate::__support::Bounds::new(&raw const __LINK_SECTION_INFO) }
        }
    }
}

pub use crate::__get_section_wasm as get_section;

crate::__def_section_name! {
    __section_name_wasm,
    {
        data bare =>    (".data", ".link_section.") __ ();
        data section => (".data", ".link_section.") __ ();
        code bare =>    (".text", ".link_section.") __ ();
        code section => (".text", ".link_section.") __ ();
        data bounds =>  (".data", ".link_section.") __ (".bounds");
    }
    AUXILIARY = ".";
    MAX_LENGTH = 16;
    HASH_LENGTH = 6;
    VALID_SECTION_CHARS = "_ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
}

#[cfg(not(target_family = "wasm"))]
#[doc(hidden)]
#[macro_export]
#[allow(unknown_lints, edition_2024_expr_fragment_specifier)]
macro_rules! __register_wasm_item {
    (value=$value:expr, $(ref=$ident:ident,)? section=$section:ident $($aux:ident)?) => {};
}

#[cfg(target_family = "wasm")]
#[doc(hidden)]
#[macro_export]
#[allow(unknown_lints, edition_2024_expr_fragment_specifier)]
macro_rules! __register_wasm_item {
    (value=$value:expr, $(ref=$ident:ident,)? section=$section:ident $($aux:ident)?) => {
        {
            // Register a counting item
            $crate::__add_section_link_attribute!(
                data section $section $($aux)?
                #[link_section = __]
                static __LINK_SECTION_COUNTING_ITEM: u8 = 0;
            );

            $crate::__add_section_link_attribute!(
                data bounds $section $($aux)?
                #[link_name = __]
                extern "C" {
                    static __LINK_SECTION_INFO: $crate::__support::wasm::LinkSectionRawInfo;
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
}

#[cfg(target_family = "wasm")]
#[allow(missing_unsafe_on_extern)] // MSRV
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
pub struct LinkSection(NonNull<LinkSectionRawInfo>);

impl LinkSection {
    /// Create a new link section.
    pub const fn new(info_ptr: NonNull<LinkSectionRawInfo>) -> Self {
        Self(info_ptr)
    }

    /// Lock the link section and return a guard.
    #[inline(always)]
    pub fn lock<'a>(&'a self) -> LinkSectionLockGuard<'a> {
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
    fn maybe_lock_uninit<'a>(&'a self, old: u8) -> LinkSectionLockGuard<'a> {
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
    unsafe fn as_mut(&self) -> &mut LinkSectionInfo {
        unsafe {
            let unsafe_cell = ptr::addr_of!((*self.0.as_ptr()).info);
            // as_mut_unchecked when we bump MSRV
            UnsafeCell::raw_get(unsafe_cell).as_mut().unwrap_unchecked()
        }
    }
}

/// Lightweight lock guard for the link section.
pub struct LinkSectionLockGuard<'a>(&'a AtomicU8, &'a mut LinkSectionInfo);
impl<'a> core::ops::Deref for LinkSectionLockGuard<'a> {
    type Target = LinkSectionInfo;
    fn deref(&self) -> &Self::Target {
        self.1
    }
}
impl<'a> core::ops::DerefMut for LinkSectionLockGuard<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.1
    }
}
impl<'a> Drop for LinkSectionLockGuard<'a> {
    fn drop(&mut self) {
        self.0.store(LockState::Unlocked as _, Ordering::Release);
    }
}

/// The current state of the link section.
#[repr(C)]
pub struct LinkSectionRawInfo {
    lock: AtomicU8,
    info: UnsafeCell<LinkSectionInfo>,
}

// SAFETY:

// Mutation of `LinkSectionInfo` is guarded by `LinkSection::lock`, which
// synchronize via `AtomicU8`.
unsafe impl Sync for LinkSectionRawInfo {}

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

impl LinkSectionRawInfo {
    /// Create a new link section raw info.
    pub const fn new<T>(name: &'static str) -> Self {
        Self {
            lock: AtomicU8::new(LockState::Uninitialized as _),
            info: UnsafeCell::new(LinkSectionInfo {
                state: LinkSectionState::Uninitialized as _,
                name_length: name.len() as _,
                name: name.as_ptr(),
                start: ptr::null_mut(),
                end: ptr::null_mut(),
                current: ptr::null_mut(),
                size_of: ::core::mem::size_of::<T>(),
                align_of: ::core::mem::align_of::<T>(),
            }),
        }
    }
}

impl LinkSectionInfo {
    /// Initialize the link section.
    pub fn initialize(&mut self) {
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
            return;
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
    }
}

/// Register a link section item.
///
/// # Safety
///
/// This is called by the `in_section` procedural macro.
pub unsafe fn register_wasm_link_section_item<T>(info_ptr: *const LinkSectionRawInfo) -> *mut T {
    let link_section = unsafe { LinkSection::new(NonNull::new_unchecked(info_ptr as _)) };
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
pub struct Bounds(LinkSection);

impl Bounds {
    /// Create a new bounds struct.
    ///
    /// # Safety
    ///
    /// This is called by the `section` procedural macro.
    pub const unsafe fn new(info_ptr: *const LinkSectionRawInfo) -> Self {
        Self(LinkSection::new(unsafe {
            NonNull::new_unchecked(info_ptr as _)
        }))
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
