#![feature(macro_metavar_expr)]
use std::alloc::{Layout, alloc, dealloc, handle_alloc_error};
use std::mem::{align_of, size_of};
use std::ptr::{self, null_mut};

#[macro_export]
macro_rules! sibling_vecs {
    ($name:ident, $($ty:ty),*) => {
        #[doc(hidden)]
        pub mod $name {
            use super::*;

            pub struct SiblingVecs {
                len: usize,
                cap: usize,
                ptr: *mut u8,
            }

            impl SiblingVecs {
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
                    let mut ma = 0;
                    $(ma = ma.max(align_of::<$ty>());)*
                    if ma == 0 { 1 } else { ma }
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

                const N: usize = $crate::__count_ty!($($ty),*);

                fn compute_offsets_with_cap(cap: usize) -> [usize; Self::N] {
                    let mut offs = [0usize; Self::N];
                    let mut current: usize = 0;
                    let mut i = 0;
                    $({
                        let a = align_of::<$ty>();
                        if a > 1 {
                            current = (current + a - 1) / a * a;
                        }
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
                    let offs = self.compute_offsets();
                    ($(
                        unsafe { ::std::slice::from_raw_parts(self.ptr.add(offs[${index(ty)}]) as *const $ty, self.len) }
                    ),*)
                }

                pub fn as_mut_slices(&mut self) -> ($(&mut [$ty]),*) {
                    let offs = self.compute_offsets();
                    ($(
                        unsafe { ::std::slice::from_raw_parts_mut(self.ptr.add(offs[$${index(ty)}]) as *mut $ty, self.len) }
                    ),*)
                }

                pub fn push(&mut self, values: ($($ty,)*)) {
                    if self.len == self.cap {
                        self.reserve();
                    }
                    let offs = self.compute_offsets();
                    $({
                        unsafe {
                            ptr::write(
                                self.ptr.add(offs[$${index(ty)}]).add(self.len * size_of::<$ty>()) as *mut $ty,
                                values.$${index(ty)}
                            );
                        }
                    })*
                    self.len += 1;
                }

                fn reserve(&mut self) {
                    let new_cap = if self.cap == 0 { 8 } else { self.cap * 2 };
                    let new_size = Self::total_size_with_cap(new_cap);
                    let align = Self::max_align();
                    let layout = Layout::from_size_align(new_size, align).unwrap();
                    let new_ptr = unsafe { alloc(layout) };
                    if new_ptr.is_null() {
                        handle_alloc_error(layout);
                    }
                    let old_offs = Self::compute_offsets_with_cap(self.cap);
                    let new_offs = Self::compute_offsets_with_cap(new_cap);
                    let len = self.len;
                    let num = Self::N;
                    for i in 0..num {
                        let idx = num - 1 - i;
                        let ty_size = size_of::<$${index(idx, ty)}>();
                        unsafe {
                            ptr::copy_nonoverlapping(
                                self.ptr.add(old_offs[idx]),
                                new_ptr.add(new_offs[idx]),
                                len * ty_size,
                            );
                        }
                    }
                    if self.cap != 0 {
                        let old_size = Self::total_size_with_cap(self.cap);
                        let old_layout = Layout::from_size_align(old_size, align).unwrap();
                        unsafe { dealloc(self.ptr, old_layout); }
                    }
                    self.ptr = new_ptr;
                    self.cap = new_cap;
                }
            }

            impl Drop for SiblingVecs {
                fn drop(&mut self) {
                    if self.cap == 0 {
                        return;
                    }
                    let offs = self.compute_offsets();
                    let len = self.len;
                    let num = Self::N;
                    for i in 0..num {
                        let idx = num - 1 - i;
                        unsafe {
                            let base = self.ptr.add(offs[idx]);
                            let ty = $${index(idx, ty)};
                            for j in (0..len).rev() {  // Drop in reverse order per column
                                ptr::drop_in_place(base.add(j * size_of::<ty>()) as *mut ty);
                            }
                        }
                    }
                    let align = Self::max_align();
                    let size = Self::total_size_with_cap(self.cap);
                    let layout = Layout::from_size_align(size, align).unwrap();
                    unsafe { dealloc(self.ptr, layout); }
                }
            }
        }

        pub use __sibling_vecs_impl_$name::SiblingVecs as $name;

        // Helper to get count (since ${count} not directly usable in const)
        #[doc(hidden)]
        macro_rules! __count_ty {
            () => { 0 };
            ($_head:ty $$(, $$_tail:ty)*) => { 1 + $crate::__count_ty!($$($$_tail),*) };
        }
    };
}
