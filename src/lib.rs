#![feature(macro_metavar_expr)]

pub use std::alloc::{Layout, alloc, dealloc, handle_alloc_error};
pub use std::mem::{align_of, size_of};
pub use std::ptr::{self, null_mut};
pub use std::slice;

#[macro_export]
macro_rules! sibling_vecs {
    ($name:ident, $($ty:ty),* $(,)?) => {
        #[doc(hidden)]
        pub mod __sibling_vecs_impl {
            use super::*;

            macro_rules! reverse_copy {
                ($self:expr, $new_ptr:expr, $old_offs:expr, $new_offs:expr, $len:expr) => {
                    reverse_copy_helper!( ; $($ty),* ; $self, $new_ptr, $old_offs, $new_offs, $len)
                };
            }

            macro_rules! reverse_copy_helper {
                ( $$($$rev:ty),* ; $curr:ty , $$($$rest:ty),* ; $self:expr, $new_ptr:expr, $old_offs:expr, $new_offs:expr, $len:expr) => {
                    reverse_copy_helper!( $curr , $$($$rev),* ; $$($$rest),* ; $self, $new_ptr, $old_offs, $new_offs, $len)
                };
                ( $$($$rev:ty),* ; ; $self:expr, $new_ptr:expr, $old_offs:expr, $new_offs:expr, $len:expr) => {
                    let n = Self::N;
                    let mut _i = 0;
                    $$(
                        let idx = n - 1 - _i;
                        unsafe { ptr::copy_nonoverlapping(
                            $self.ptr.add($old_offs[idx]),
                            $new_ptr.add($new_offs[idx]),
                            $len * size_of::<$rev>(),
                        ); }
                        _i += 1;
                    )*
                };
            }

            macro_rules! reverse_drop {
                ($self:expr, $offs:expr, $len:expr) => {
                    reverse_drop_helper!( ; $($ty),* ; $self, $offs, $len)
                };
            }

            macro_rules! reverse_drop_helper {
                ( $$($$rev:ty),* ; $curr:ty , $$($$rest:ty),* ; $self:expr, $offs:expr, $len:expr) => {
                    reverse_drop_helper!( $curr , $$($$rev),* ; $$($$rest),* ; $self, $offs, $len)
                };
                ( $$($$rev:ty),* ; ; $self:expr, $offs:expr, $len:expr) => {
                    let n = Self::N;
                    let mut _i = 0;
                    $$(
                        let idx = n - 1 - _i;
                        unsafe {
                            let base = $self.ptr.add($offs[idx]);
                            for j in 0..$len {
                                ptr::drop_in_place(base.add(j * size_of::<$rev>()) as *mut $rev);
                            }
                        }
                        _i += 1;
                    )*
                };
            }

            pub struct $name {
                len: usize,
                cap: usize,
                ptr: *mut u8,
            }

            impl $name {
                pub fn new() -> Self {
                    Self::with_capacity(0)
                }

                pub fn with_capacity(cap: usize) -> Self {
                    if cap == 0 {
                        return Self { len: 0, cap: 0, ptr: null_mut() };
                    }
                    let size = Self::total_size_with_cap(cap);
                    let align = Self::max_align();
                    let layout = Layout::from_size_align(size, align).unwrap();
                    let ptr = unsafe { alloc(layout) };
                    if ptr.is_null() {
                        handle_alloc_error(layout);
                    }
                    Self { len: 0, cap, ptr }
                }

                fn max_align() -> usize {
                    let mut ma = 1;
                    $(ma = ma.max(align_of::<$ty>());)*
                    ma
                }

                fn total_size_with_cap(cap: usize) -> usize {
                    let mut current = 0;
                    $({
                        let a = align_of::<$ty>();
                        current = (current + a - 1) / a * a;
                        current += cap * size_of::<$ty>();
                    })*
                    current
                }

                const N: usize = ${count($ty)};

                fn compute_offsets_with_cap(cap: usize) -> [usize; Self::N] {
                    let mut offs = [0; Self::N];
                    let mut current = 0;
                    let mut i = 0;
                    $({
                        let a = align_of::<$ty>();
                        current = (current + a - 1) / a * a;
                        offs[i] = current;
                        current += cap * size_of::<$ty>();
                        i += 1;
                    })*
                    offs
                }

                fn compute_offsets(&self) -> [usize; Self::N] {
                    Self::compute_offsets_with_cap(self.cap)
                }

                pub fn as_slices(&self) -> ($(&[$ty]),*) {
                    if self.len == 0 {
                        return ($(&[] as &[$ty]),*);
                    }
                    let offs = self.compute_offsets();
                    ($(
                        unsafe { slice::from_raw_parts(self.ptr.add(offs[${index()}]) as *const $ty, self.len) }
                    ),*)
                }

                pub fn as_mut_slices(&mut self) -> ($(&mut [$ty]),*) {
                    if self.len == 0 {
                        return ($(&mut [] as &mut [$ty]),*);
                    }
                    let offs = self.compute_offsets();
                    ($(
                        unsafe { slice::from_raw_parts_mut(self.ptr.add(offs[${index()}]) as *mut $ty, self.len) }
                    ),*)
                }

                pub fn push(&mut self, values: ($($ty,)*)) {
                    if self.len == self.cap {
                        self.reserve();
                    }
                    let offs = self.compute_offsets();
                    let pos = self.len;
                    $(unsafe {
                        self.ptr.add(offs[${index()}]).add(pos * size_of::<$ty>()).cast::<$ty>().write(values.${index()});
                    })*
                    self.len += 1;
                }

                fn reserve(&mut self) {
                    let new_cap = if self.cap == 0 { 4 } else { self.cap * 2 };
                    let new_size = Self::total_size_with_cap(new_cap);
                    let align = Self::max_align();
                    let layout = Layout::from_size_align(new_size, align).unwrap();
                    let new_ptr = unsafe { alloc(layout) };
                    if new_ptr.is_null() {
                        handle_alloc_error(layout);
                    }
                    let old_offs = Self::compute_offsets_with_cap(self.cap);
                    let new_offs = Self::compute_offsets_with_cap(new_cap);
                    reverse_copy!(self, new_ptr, old_offs, new_offs, self.len);
                    if self.cap != 0 {
                        let old_size = Self::total_size_with_cap(self.cap);
                        let old_layout = Layout::from_size_align(old_size, align).unwrap();
                        unsafe { dealloc(self.ptr, old_layout); }
                    }
                    self.ptr = new_ptr;
                    self.cap = new_cap;
                }
            }

            impl Drop for $name {
                fn drop(&mut self) {
                    if self.len == 0 || self.cap == 0 {
                        return;
                    }
                    let offs = self.compute_offsets();
                    reverse_drop!(self, offs, self.len);
                    let align = Self::max_align();
                    let size = Self::total_size_with_cap(self.cap);
                    let layout = Layout::from_size_align(size, align).unwrap();
                    unsafe { dealloc(self.ptr, layout); }
                }
            }
        }
    };
}
