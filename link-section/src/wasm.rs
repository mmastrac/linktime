//! WASM-specific implementation of the link section.
use alloc::alloc::alloc;
use core::alloc::Layout;
use core::ptr::{self, NonNull};
use core::sync::atomic::{AtomicU8, Ordering};

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
    pub const fn new(info_ptr: NonNull<LinkSectionRawInfo>) -> Self {
        Self(info_ptr)
    }

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
        if old == LockState::Uninitialized as _ {
            if let Err(_) = lock_state.compare_exchange(
                LockState::Uninitialized as _,
                LockState::Locked as _,
                Ordering::Acquire,
                Ordering::Acquire,
            ) {
                panic!("Link section already being initialized");
            }
            let info = unsafe { self.as_mut() };
            info.initialize();
            return LinkSectionLockGuard(lock_state, info);
        } else {
            panic!("Link section already locked");
        }
    }

    #[inline(always)]
    unsafe fn lock_ref(&self) -> &AtomicU8 {
        unsafe { ptr::addr_of!((*self.0.as_ptr()).lock).as_ref_unchecked() }
    }

    #[inline(always)]
    unsafe fn as_mut(&self) -> &mut LinkSectionInfo {
        unsafe { ptr::addr_of_mut!((*self.0.as_ptr()).info).as_mut_unchecked() }
    }
}

/// Lightweight lock guard for the link section.
pub struct LinkSectionLockGuard<'a>(&'a AtomicU8, &'a mut LinkSectionInfo);
impl<'a> core::ops::Deref for LinkSectionLockGuard<'a> {
    type Target = LinkSectionInfo;
    fn deref(&self) -> &Self::Target {
        &self.1
    }
}
impl<'a> core::ops::DerefMut for LinkSectionLockGuard<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.1
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
    info: LinkSectionInfo,
}

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
    pub const fn new<T>(name: &'static str) -> Self {
        Self {
            lock: AtomicU8::new(LockState::Uninitialized as _),
            info: LinkSectionInfo {
                state: LinkSectionState::Uninitialized as _,
                name_length: name.len() as _,
                name: name.as_ptr(),
                start: ptr::null_mut(),
                end: ptr::null_mut(),
                current: ptr::null_mut(),
                size_of: ::core::mem::size_of::<T>(),
                align_of: ::core::mem::align_of::<T>(),
            },
        }
    }
}

impl LinkSectionInfo {
    pub fn initialize(&mut self) {
        let size = unsafe {
            crate::__support::read_custom_section(
                self.name,
                self.name_length as _,
                ptr::null_mut(),
                0,
            )
        };

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
                alloc(Layout::from_size_align(layout_bytes, self.align_of).unwrap_unchecked());
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

pub unsafe fn register_wasm_link_section_item<T>(info_ptr: *mut LinkSectionRawInfo) -> *mut T {
    let link_section = LinkSection::new(NonNull::new_unchecked(info_ptr));
    let mut info = link_section.lock();

    unsafe {
        if info.state == LinkSectionState::Initialized as _ {
            panic!("Link section already initialized");
        }

        let slot = info.current;
        let next = slot.byte_add(info.size_of) as *const ();
        if next > info.end {
            panic!("Link section overflow: too many registered items");
        }

        info.current = next;
        if next == info.end {
            info.state = LinkSectionState::Initialized as _;
        }
        slot as *mut T
    }
}

/// On WASM, we use an atomic pointer to the start and end of the
/// section. The host environment is responsible for registering the
/// section with the runtime.
pub struct Bounds(LinkSection);

impl Bounds {
    pub const unsafe fn new(info_ptr: *mut LinkSectionRawInfo) -> Self {
        Self(LinkSection::new(unsafe {
            NonNull::new_unchecked(info_ptr)
        }))
    }

    pub fn start_ptr(&self) -> *const () {
        let lock = self.0.lock();
        if lock.state != LinkSectionState::Initialized as _ {
            panic!("Link section not initialized: possible ctor ordering issue");
        }
        return lock.start;
    }

    pub fn end_ptr(&self) -> *const () {
        let lock = self.0.lock();
        if lock.state != LinkSectionState::Initialized as _ {
            panic!("Link section not initialized: possible ctor ordering issue");
        }
        return lock.end;
    }

    /// This is intentionally safe to call before the section is fully
    /// initialized.
    pub fn byte_len(&self) -> usize {
        let lock = self.0.lock();
        return unsafe { lock.end.byte_offset_from(lock.start) as usize };
    }
}
