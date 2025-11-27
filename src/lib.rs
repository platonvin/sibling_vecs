#![feature(macro_metavar_expr)]

#[macro_export]
macro_rules! sibling_vecs {
    (
        $vis:vis struct $name:ident {
            $( $field:ident : $type:ty ),* $(,)?
        }
    ) => {
        // Main struct for all sibling sub-vecs.
        $vis struct $name {
            ptr: *mut u8,
            len: usize,
            cap: usize,
        }

        impl $name {
            pub const N: usize = ${count($type)};

            pub fn new() -> Self {
                Self {
                    ptr: std::ptr::null_mut(),
                    len: 0,
                    cap: 0,
                }
            }
            pub fn with_capacity(cap: usize) -> Self {
                let mut s = Self::new();
                s.reallocate_to(cap);
                s
            }

            pub fn len(&self) -> usize {
                self.len
            }
            pub fn capacity(&self) -> usize {
                self.cap
            }

            const fn type_infos() -> [(usize, usize); Self::N] {
                [ $( (std::mem::size_of::<$type>(), std::mem::align_of::<$type>()) ),* ]
            }

            fn offsets(cap: usize) -> [usize; Self::N] {
                let infos = Self::type_infos();
                let mut out = [0; Self::N];
                let mut current_offset = 0;

                let mut i = 0;
                // TODO: should we manually unroll with macro? Where does it start optimizing away?
                while i < Self::N {
                    let (size, align) = infos[i];

                    if align > 0 {
                        let remainder = current_offset % align;
                        if remainder != 0 {
                            current_offset += align - remainder;
                        }
                    }

                    out[i] = current_offset;
                    current_offset += cap * size;
                    i += 1;
                }
                out
            }

            fn layout(cap: usize) -> std::alloc::Layout {
                if cap == 0 {
                    return std::alloc::Layout::new::<u8>();
                }

                let infos = Self::type_infos();
                let offsets = Self::offsets(cap);

                let last_idx = Self::N - 1;
                let (last_size, _) = infos[last_idx];
                let total_size = offsets[last_idx] + (cap * last_size);

                let mut max_align = 1;
                let mut i = 0;
                while i < Self::N {
                    let (_, align) = infos[i];
                    if align > max_align { max_align = align; }
                    i += 1;
                }

                std::alloc::Layout::from_size_align(total_size, max_align).unwrap()
            }

            // Helper function for allocation-related stuff.
            fn reallocate_to(&mut self, new_cap: usize) {
                let old_cap = self.cap;
                if new_cap == old_cap { return; }

                // deallocate
                if new_cap == 0 {
                    if old_cap > 0 {
                        unsafe {
                            std::alloc::dealloc(self.ptr, Self::layout(old_cap));
                        }
                    }
                    self.ptr = std::ptr::null_mut();
                    self.cap = 0;
                    self.len = 0;
                    return;
                }

                let old_layout = Self::layout(old_cap);
                let new_layout = Self::layout(new_cap);

                unsafe {
                    // alloc or realloc
                    let new_ptr = if old_cap == 0 {
                        std::alloc::alloc(new_layout)
                    } else {
                        // TODO: realloc nullptr?
                        std::alloc::realloc(self.ptr, old_layout, new_layout.size())
                    };

                    if new_ptr.is_null() { std::alloc::handle_alloc_error(new_layout); }

                    self.ptr = new_ptr;
                    self.cap = new_cap;
                    // TODO: thing is, if we stay in-memory we do actually want shifting
                    // but when we cant, and realloc would move, we would rather avoid copy-all-then-move and straight up copy once but properly

                    if self.len > 0 && old_cap > 0 {
                        let old_offsets = Self::offsets(old_cap);
                        let new_offsets = Self::offsets(new_cap);
                        let infos = Self::type_infos();

                        // shift data (if realloc)
                        // reverse because otherwise we will overwrite data of next sub-vec
                        // from 1 cause 0th does not need to be shifted
                        for i in (1..Self::N).rev() {
                            let (size, _) = infos[i];
                            let size_bytes = self.len * size;

                            let src = self.ptr.add(old_offsets[i]);
                            let dst = self.ptr.add(new_offsets[i]);

                            std::ptr::copy(src, dst, size_bytes);
                        }
                    }
                }
            }

            fn grow(&mut self) {
                // does not actually matter, it is intended to never really shrink
                let new_cap = if self.cap == 0 { 4 } else { self.cap * 2 };
                self.reallocate_to(new_cap);
            }

            pub fn push(&mut self, $( $field : $type ),* ) {
                if self.len == self.cap {
                    self.grow();
                }

                let offsets = Self::offsets(self.cap);

                unsafe {
                    $(
                        // since we are inside a repetition $()* for methods,
                        // ${index()} gives us the index of the current iteration
                        let offset = offsets[${index()}];
                        let type_base = self.ptr.add(offset) as *mut $type;
                        type_base.add(self.len).write($field);
                    )*
                }
                self.len += 1;
            }

            // nice thing is that there is no bounds checking and its up to user
            pub fn as_slices(&self) -> ( $( &[$type] ),* ) {
                let offsets = Self::offsets(self.cap);
                unsafe {
                    (
                        $(
                            std::slice::from_raw_parts(
                                self.ptr.add(offsets[${index()}]) as *const $type,
                                self.len
                            )
                        ),*
                    )
                }
            }

            // nice thing is that there is no bounds checking and its up to user
            pub fn as_mut_slices(&mut self) -> ( $( &mut [$type] ),* ) {
                let offsets = Self::offsets(self.cap);
                unsafe {
                    (
                        $(
                            std::slice::from_raw_parts_mut(
                                self.ptr.add(offsets[${index()}]) as *mut $type,
                                self.len
                            )
                        ),*
                    )
                }
            }

            $(
                pub fn $field(&self) -> &[$type] {
                     let offsets = Self::offsets(self.cap);
                     let idx = ${index()};
                     unsafe {
                         std::slice::from_raw_parts(
                             self.ptr.add(offsets[idx]) as *const $type,
                             self.len
                         )
                     }
                }
            )*
        }

        impl Drop for $name {
            fn drop(&mut self) {
                if self.cap > 0 {
                    debug_assert!(!self.ptr.is_null());
                    let offsets = Self::offsets(self.cap);

                    unsafe {
                        $(
                            // should be fine to just drop but anyways
                            if std::mem::needs_drop::<$type>() {
                                let offset = offsets[${index()}];
                                let base = self.ptr.add(offset) as *mut $type;
                                for i in 0..self.len {
                                    std::ptr::drop_in_place(base.add(i));
                                }
                            }
                        )*
                        std::alloc::dealloc(self.ptr, Self::layout(self.cap));
                    }
                }
            }
        }
    };
}
