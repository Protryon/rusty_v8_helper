use rusty_v8 as v8;
use std::any::Any;
use std::any::TypeId;
use std::cell::RefCell;
use std::ffi::c_void;
use std::hash::Hash;
use std::hash::Hasher;
use std::ptr::NonNull;
use std::rc::Rc;
use v8::Global;
use v8::InIsolate;
use v8::Isolate;
use v8::IsolateHandle;
use v8::Local;
use v8::Object;
use v8::ToLocal;
use v8::{WeakCallback, Weakable};

/// `ObjectWrap` is a non-standard helper to match arbitrary Rust objects
/// to arbitrary JS objects within V8.
///
/// The `ObjectWrap` and the wrapped `T` are reference counted, and `T` is
/// deallocated by the V8 GC once all references have fallen out of scope.
///
/// If the V8-facing JS object has been deallocated, then all methods on
/// `ObjectWrap` will return `None`, `false`, or do nothing.
///
/// Similarly, when dropped, if the V8-facing JS object has been deallocated
/// then the `ObjectWrap` is dropped but no other change happens.
/// If the V8-facing JS object has not been dropped, it is impossible for the
/// `ObjectWrap` to be dropped, as it has a reference existing in the V8 GC.
///
/// In order for the V8 GC to track this object to be deallocated is to call
/// `ObjectWrap::make_weak`. You can disable GC tracking with
/// `ObjectWrap::clear_weak`.
#[derive(Clone)]
pub struct ObjectWrap<T: Any + 'static>(Rc<ObjectWrapInternal<T>>);

struct ObjectWrapInternal<T: Any + 'static> {
    handle: RefCell<Option<Global<Object>>>,
    wrapping: RefCell<Option<*const T>>,
    v8_reference: RefCell<Option<*const Self>>,
    isolate_handle: IsolateHandle,
}

unsafe impl<T: 'static, Y: Any + 'static> Weakable<T> for ObjectWrapInternal<Y> {
    fn get(self: Rc<Self>, _global: &Global<T>) -> NonNull<c_void> {
        let v8_reference = Rc::into_raw(self.clone());
        assert_eq!(self.v8_reference.replace(Some(v8_reference)), None);
        unsafe { NonNull::new_unchecked(v8_reference as *mut libc::c_void) }
    }

    fn clear(&self, _global: &Global<T>) {
        unsafe { Rc::from_raw(self.v8_reference.borrow_mut().take().unwrap()) };
    }

    fn get_callback(&self, _global: &Global<T>) -> WeakCallback<c_void> {
        wrap_weak_callback::<Y>
    }
}

// only reads first 8 bytes
#[derive(Default)]
struct TypeIdHasher {
    bytes_read: [u8; 8],
    index: usize,
}

impl Hasher for TypeIdHasher {
    fn write(&mut self, bytes: &[u8]) {
        if self.index >= 8 {
            return;
        }
        self.bytes_read
            .copy_from_slice(&bytes[0..bytes.len().max(8 - self.index)]);
    }

    fn finish(&self) -> u64 {
        u64::from_le_bytes(self.bytes_read)
    }
}

// TODO: come up with more elegant way to get a type_id in V8 without a dereference
fn type_id_to_u64<T: Any + 'static>() -> u64 {
    let mut hasher = TypeIdHasher::default();
    TypeId::of::<T>().hash(&mut hasher);
    hasher.finish() & (!1_u64) // must be 2 byte aligned
}

impl<T: Any + 'static> ObjectWrap<T> {
    /// Create a new `ObjectWrap` from a given scope, an `Object` that
    /// has exactly 1 allocated internal fields through
    /// `ObjectTemplate::set_internal_field_count`, and an arbitrary
    /// `T` to tag with the Object.
    pub fn new(scope: &mut impl InIsolate, mut object: Local<Object>, wrap: T) -> ObjectWrap<T> {
        assert_eq!(object.internal_field_count(), 2);
        let wrap = Rc::into_raw(Rc::new(wrap));
        unsafe { object.set_internal_field_ptr(0, type_id_to_u64::<T>() as usize as *mut c_void) };
        unsafe { object.set_internal_field_ptr(1, wrap as *mut T) };
        let mut global = Global::new_from(scope, object);
        let wrapper = ObjectWrap(Rc::new(ObjectWrapInternal {
            handle: RefCell::new(None),
            wrapping: RefCell::new(Some(wrap)),
            v8_reference: RefCell::new(None),
            isolate_handle: IsolateHandle::new(scope.isolate()),
        }));
        global.set_weakable(wrapper.0.clone());
        wrapper.0.handle.replace(Some(global));
        wrapper
    }

    /// Resolves an arbitrary `Object` to a `std::rc::Rc<T>` if it has a valid type.
    ///
    /// Otherwise, returns None.
    pub fn from_object(object: Local<Object>) -> Option<Rc<T>> {
        if object.internal_field_count() != 2 {
            return None;
        }
        let expected_type_id = type_id_to_u64::<T>() as usize;
        let actual_type_id = unsafe { object.get_internal_field_ptr::<c_void>(0) } as usize;
        if expected_type_id != actual_type_id {
            return None;
        }
        let raw_ptr = unsafe { object.get_internal_field_ptr::<T>(1) };
        let temp_rc = unsafe { Rc::from_raw(raw_ptr as *const T) };
        let new_rc = temp_rc.clone();
        Rc::into_raw(temp_rc);
        Some(new_rc)
    }

    /// Get the underlying `Object` that is represented by this `ObjectWrap`.
    pub fn get<'sc>(&self, scope: &mut impl ToLocal<'sc>) -> Option<Local<'sc, Object>> {
        self.0.handle.borrow().as_ref()?.get(scope)
    }

    /// Unwrap a `std::rc::Rc<T>` wrapped by this `ObjectWrap`.
    pub fn unwrap<'sc>(&self, scope: &mut impl ToLocal<'sc>) -> Option<Rc<T>> {
        let object = self.0.handle.borrow().as_ref()?.get(scope)?;

        let wrapped_ptr = unsafe { object.get_internal_field_ptr(1) } as *const T;
        let rc = unsafe { Rc::from_raw(wrapped_ptr) };
        let new_rc = rc.clone();
        Rc::into_raw(rc);
        Some(new_rc)
    }

    /// Swap the `T` wrapped by this `ObjectWrap` with another.
    /// Note that existing references to the `T` previously in this `ObjectWrap`
    /// will continue to hold onto the value through a reference counter.
    pub fn swap<'sc>(&mut self, scope: &mut impl ToLocal<'sc>, wrap: T) -> Option<Rc<T>> {
        let mut object = self.0.handle.borrow().as_ref()?.get(scope)?;
        if object.internal_field_count() != 2 {
            return None;
        }

        let wrapped_ptr = unsafe { object.get_internal_field_ptr(1) } as *mut T;
        let wrapped = unsafe { Rc::from_raw(wrapped_ptr) };
        let new_ptr = Rc::into_raw(Rc::new(wrap));
        self.0.wrapping.replace(Some(new_ptr));
        unsafe { object.set_internal_field_ptr(1, new_ptr as *mut T) }

        Some(wrapped)
    }

    /// Enable V8 GC to collect the `Object` represented by this `ObjectWrap`.
    pub fn make_weak(&mut self) {
        if let Some(global) = self.0.handle.borrow_mut().as_mut() {
            global.set_weak();
        }
    }

    /// Check if V8 GC collection is enabled for this `ObjectWrap`.
    ///
    /// `false` if the object has been deallocated.
    pub fn is_weak(&self) -> bool {
        if let Some(global) = self.0.handle.borrow_mut().as_mut() {
            global.is_weak()
        } else {
            false
        }
    }

    /// Disable V8 GC from deallocating the `Object` represented by this
    /// `ObjectWrap`.
    pub fn clear_weak(&mut self) {
        if let Some(global) = self.0.handle.borrow_mut().as_mut() {
            global.clear_weak();
        }
    }

    /// Check if the `Object` represented by this `ObjectWrap` has been
    /// deallocated.
    pub fn is_empty(&self) -> bool {
        self.0.handle.borrow().is_none()
    }
}

impl<T> Drop for ObjectWrapInternal<T> {
    fn drop(&mut self) {
        let isolate = unsafe { self.isolate_handle.get_isolate_ptr().as_mut() };
        if isolate.is_none() {
            return;
        }
        let isolate = isolate.unwrap();
        let handle = &mut self.handle.borrow_mut();
        if handle.is_none() {
            return;
        }
        let mut handle = handle.take().unwrap();
        if handle.is_weak() {
            handle.clear_weak();
        }
        let object = handle.get_isolate(isolate);
        if object.is_none() {
            return;
        }
        let object = object.unwrap();
        let wrapped_ptr = unsafe { object.get_internal_field_ptr(1) } as *mut T;
        self.wrapping.borrow_mut().take();
        unsafe { Rc::from_raw(wrapped_ptr) };
    }
}

extern "C" fn wrap_weak_callback<T: 'static>(
    value: NonNull<c_void>,
    mut isolate: NonNull<Isolate>,
) {
    let this = unsafe {
        (&value as *const NonNull<c_void> as *mut NonNull<ObjectWrapInternal<T>>)
            .as_mut()
            .unwrap()
            .as_ref()
    };
    let this = unsafe { Rc::from_raw(this) };
    let isolate = unsafe { isolate.as_mut() };

    let mut handle = this.handle.borrow_mut();
    if handle.is_none() {
        return;
    }
    let mut handle = handle.take().unwrap();
    handle.set_isolate(isolate, None);

    let ref_ptr = this.wrapping.borrow_mut().take();
    if let Some(ref_ptr) = ref_ptr {
        drop(unsafe { Rc::from_raw(ref_ptr) });
    }
}
