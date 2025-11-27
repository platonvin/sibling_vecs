#![allow(unused)]
#![feature(macro_metavar_expr)]

#[macro_export]
macro_rules! sibling_vecs {
    ($name:ident, $($ty:ty),* $(,)?) => {
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
                    return Self { len: 0, cap: 0, ptr: std::ptr::null_mut() };
                }
                let size = Self::total_size_with_cap(cap);
                let align = Self::max_align();
                let layout = std::alloc::Layout::from_size_align(size, align).unwrap();
                let ptr = unsafe { std::alloc::alloc(layout) };
                if ptr.is_null() {
                    std::alloc::handle_alloc_error(layout);
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
                    if a > 0 {
                        current = (current + a - 1) / a * a;
                    }
                    current += cap * size_of::<$ty>();
                })*
                current
            }

            fn compute_offsets_with_cap(cap: usize) -> Vec<usize> {
                let mut offs = Vec::new();
                let mut current = 0;
                $({
                    let a = align_of::<$ty>();
                    if a > 0 {
                        current = (current + a - 1) / a * a;
                    }
                    offs.push(current);
                    current += cap * size_of::<$ty>();
                })*
                offs
            }

            fn compute_offsets(&self) -> Vec<usize> {
                Self::compute_offsets_with_cap(self.cap)
            }

            pub fn as_slices(&self) -> ($(&[$ty]),*) {
                if self.len == 0 {
                    return ($( &[] as &[$ty] ),*);
                }
                let offs = self.compute_offsets();
                sibling_vecs!(@gen_slices, self, offs, 0, $($ty),*)
            }

            pub fn as_mut_slices(&mut self) -> ($(&mut [$ty]),*) {
                if self.len == 0 {
                    return ($( &mut [] as &mut [$ty] ),*);
                }
                let offs = self.compute_offsets();
                sibling_vecs!(@gen_mut_slices, self, offs, 0, $($ty),*)
            }

            pub fn push(&mut self, values: ($($ty,)*)) {
                if self.len == self.cap {
                    self.reserve();
                }
                let offs = self.compute_offsets();
                sibling_vecs!(@write_values, self, offs, self.len, values, 0, $($ty),*);
                self.len += 1;
            }

            fn reserve(&mut self) {
                let new_cap = if self.cap == 0 { 4 } else { self.cap * 2 };
                let new_size = Self::total_size_with_cap(new_cap);
                let align = Self::max_align();
                let layout = std::alloc::Layout::from_size_align(new_size, align).unwrap();
                let new_ptr = unsafe { std::alloc::alloc(layout) };
                if new_ptr.is_null() {
                    std::alloc::handle_alloc_error(layout);
                }
                let old_offs = Self::compute_offsets_with_cap(self.cap);
                let new_offs = Self::compute_offsets_with_cap(new_cap);
                sibling_vecs!(@reverse_copy, self, new_ptr, old_offs, new_offs, self.len, $($ty),*);
                if self.cap != 0 {
                    let old_size = Self::total_size_with_cap(self.cap);
                    let old_layout = std::alloc::Layout::from_size_align(old_size, align).unwrap();
                    unsafe { std::alloc::dealloc(self.ptr, old_layout); }
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
                sibling_vecs!(@reverse_drop, self, offs, self.len, $($ty),*);
                let align = Self::max_align();
                let size = Self::total_size_with_cap(self.cap);
                let layout = std::alloc::Layout::from_size_align(size, align).unwrap();
                unsafe { std::alloc::dealloc(self.ptr, layout); }
            }
        }

        // Helpers

        #[allow(unused_macros)]
        #[macro_use]
        macro_rules! __gen_slices_helper {
            ($self:expr, $offs:expr, $idx:expr, $curr:ty, $$($$rest:ty),*) => {
                unsafe { slice::from_raw_parts($self.ptr.add($offs[$idx]) as *const $curr, $self.len) },
                sibling_vecs!(@gen_slices, $self, $offs, $idx + 1, $$($$rest),*)
            };
            ($self:expr, $offs:expr, $idx:expr,) => {};
        }

        // Similar for mut
        #[allow(unused_macros)]
        #[macro_use]
        macro_rules! __gen_mut_slices_helper {
            ($self:expr, $offs:expr, $idx:expr, $curr:ty, $$($$rest:ty),*) => {
                unsafe { slice::from_raw_parts_mut($self.ptr.add($offs[$idx]) as *mut $curr, $self.len) },
                sibling_vecs!(@gen_mut_slices, $self, $offs, $idx + 1, $$($$rest),*)
            };
            ($self:expr, $offs:expr, $idx:expr,) => {};
        }

        // For write
        #[allow(unused_macros)]
        #[macro_use]
        macro_rules! __write_values_helper {
            ($self:expr, $offs:expr, $pos:expr, $values:expr, $idx:expr, $curr:ty, $$($$rest:ty),*) => {
                unsafe { $self.ptr.add($offs[$idx]).add($pos * size_of::<$curr>()).cast::<$curr>().write($values.$idx); }
                sibling_vecs!(@write_values, $self, $offs, $pos, $values, $idx + 1, $$($$rest),*)
            };
            ($self:expr, $offs:expr, $pos:expr, $values:expr, $idx:expr,) => {};
        }

        // For reverse copy
        #[allow(unused_macros)]
        #[macro_use]
        macro_rules! __reverse_copy_helper {
            ($$($$rev:ty),* ; $curr:ty , $$($$rest:ty),* ; $self:expr, $new_ptr:expr, $old_offs:expr, $new_offs:expr, $len:expr) => {
                sibling_vecs!(@reverse_copy_helper, $curr, $$($$rev),* ; $$($$rest),* ; $self, $new_ptr, $old_offs, $new_offs, $len)
            };
            ($$($$rev:ty),* ; ; $self:expr, $new_ptr:expr, $old_offs:expr, $new_offs:expr, $len:expr) => {
                let n = $old_offs.len();
                let mut _i = 0;
                $$({
                    let idx = n - 1 - _i;
                    unsafe { ptr::copy($self.ptr.add($old_offs[idx]), $new_ptr.add($new_offs[idx]), $len * size_of::<$$rev>()); }
                    _i += 1;
                })*
            };
        }

        // For reverse drop
        #[allow(unused_macros)]
        #[macro_use]
        macro_rules! __reverse_drop_helper {
            ($$($$rev:ty),* ; $curr:ty , $$($$rest:ty),* ; $self:expr, $offs:expr, $len:expr) => {
                sibling_vecs!(@reverse_drop_helper, $curr, $$($$rev),* ; $$($$rest),* ; $self, $offs, $len)
            };
            ($$($$rev:ty),* ; ; $self:expr, $offs:expr, $len:expr) => {
                let n = $offs.len();
                let mut _i = 0;
                $$({
                    let idx = n - 1 - _i;
                    unsafe {
                        for j in 0..$len {
                            ptr::drop_in_place($self.ptr.add($offs[idx]).add(j * size_of::<$rev>()) as *mut $$rev);
                        }
                    }
                    _i += 1;
                })*
            };
        }

        // Dispatchers
        #[macro_use]
        #[macro_export]
        macro_rules! __dispatch {
            (@gen_slices, $self:expr, $offs:expr, $idx:expr, $$($$gens:ty),*) => { __gen_slices_helper!($self, $offs, $idx, $$($$gens),*) };
            (@gen_mut_slices, $self:expr, $offs:expr, $idx:expr, $$($$gens:ty),*) => { __gen_mut_slices_helper!($self, $offs, $idx, $$($$gens),*) };
            (@write_values, $self:expr, $offs:expr, $pos:expr, $values:expr, $idx:expr, $$($$gens:ty),*) => { __write_values_helper!($self, $offs, $pos, $values, $idx, $$($$gens),*) };
            (@reverse_copy, $self:expr, $new_ptr:expr, $old_offs:expr, $new_offs:expr, $len:expr, $$($$gens:ty),*) => { __reverse_copy_helper!( ; $$($$gens),* ; $self, $new_ptr, $old_offs, $new_offs, $len) };
            (@reverse_drop, $self:expr, $offs:expr, $len:expr, $$($$gens:ty),*) => { __reverse_drop_helper!( ; $$($$gens),* ; $self, $offs, $len) };
        }
        #[macro_use]
        #[macro_export]
        use __dispatch as sibling_vecs;
    };
}
