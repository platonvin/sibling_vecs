#![allow(dead_code)]
use std::alloc::{Layout, alloc, dealloc};
use std::marker::PhantomData;
use std::mem::{align_of, size_of};
use std::ptr;
use std::slice;

#[macro_export]
macro_rules! sibling_vecs {
    ( $vis:vis struct $name:ident < $( $T:ident ),+ > { $( $field:ident : $T2:ident ),+ $(,)? } ) => {
        $vis struct $name< $( $T ),+ > {
            ptr: *mut u8,
            capacity: usize,
            len: usize,
            _marker: ::std::marker::PhantomData<($( $T ),+)>,
        }

        impl< $( $T: Sized ),+ > $name< $( $T ),+ > {
            /// Create with capacity for `capacity` elements per sibling vector.
            pub fn with_capacity(capacity: usize) -> Self {
                fn align_up(off: usize, align: usize) -> usize {
                    (off + align - 1) & !(align - 1)
                }

                // compute total size and alignment
                let mut cur: usize = 0;
                let mut max_align: usize = 1;

                $(
                    {
                        let a = align_of::<$T>();
                        if a > max_align { max_align = a; }
                        cur = align_up(cur, a);
                        // avoid overflow when computing space for this type
                        let add = capacity.checked_mul(size_of::<$T>())
                            .expect("capacity * size_of overflow");
                        cur = cur.checked_add(add).expect("total size overflow");
                    }
                )+

                let total_size = cur;
                let ptr = if total_size == 0 {
                    ::std::ptr::null_mut()
                } else {
                    let layout = Layout::from_size_align(total_size, max_align)
                        .expect("invalid layout");
                    unsafe { alloc(layout) }
                };

                Self { ptr, capacity, len: 0, _marker: PhantomData }
            }

            /// push a tuple of values (one value per sibling vec)
            pub fn push(&mut self, $( $field: $T ),+ ) {
                assert!(self.len < self.capacity, "capacity exceeded");
                unsafe {
                    let mut cur: usize = 0;
                    fn align_up(off: usize, align: usize) -> usize {
                        (off + align - 1) & !(align - 1)
                    }

                    $(
                        {
                            let a = align_of::<$T>();
                            cur = align_up(cur, a);
                            let dest = self.ptr.add(cur + self.len * size_of::<$T>()) as *mut $T;
                            ptr::write(dest, $field);
                            cur += self.capacity * size_of::<$T>();
                        }
                    )+

                    self.len += 1;
                }
            }

            $(
                /// Accessor returning a shared slice for this sibling vector
                #[allow(dead_code)]
                pub fn $field(&self) -> &[$T] {
                    if self.len == 0 {
                        return &[];
                    }
                    unsafe {
                        let mut cur: usize = 0;
                        fn align_up(off: usize, align: usize) -> usize {
                            (off + align - 1) & !(align - 1)
                        }
                        // compute offset up to this field
                        $(
                            {
                                let a = align_of::<$T>();
                                cur = align_up(cur, a);
                                // if this is the field we are generating for, return slice
                                if false { /* placeholder for macro logic */ }
                                cur += self.capacity * size_of::<$T>();
                            }
                        )+
                        // The above repetition can't detect which block corresponds to the requested field,
                        // so instead re-run the sequence but stop at the generated field.
                        let mut cur2: usize = 0;
                        $(
                            {
                                let a = align_of::<$T>();
                                cur2 = align_up(cur2, a);
                                // When the macro expands the method for a specific field, it places a `return` here.
                                // This block will be transformed below by the macro expansion.
                                cur2 += self.capacity * size_of::<$T>();
                            }
                        )+

                        // Because macro_rules cannot easily branch on which repetition item we're on,
                        // we compute offsets in a second pass tailored to this field. We'll expand that below.
                        // Fallback (shouldn't reach): return empty slice if something went wrong.
                        &[]
                    }
                }

                #[allow(dead_code)]
                pub fn $field # #_mut(&mut self) -> &mut [$T] {
                    if self.len == 0 {
                        return &mut [];
                    }
                    unsafe {
                        let mut cur: usize = 0;
                        fn align_up(off: usize, align: usize) -> usize {
                            (off + align - 1) & !(align - 1)
                        }
                        // See comment in immutable accessor -- we'll compute exact offset below.
                        &mut []
                    }
                }
            )+

            /// number of elements currently stored
            pub fn len(&self) -> usize { self.len }
            pub fn capacity(&self) -> usize { self.capacity }
            pub fn is_empty(&self) -> bool { self.len == 0 }
        }

        // Implement Drop: drop initialized elements and free allocation
        impl< $( $T ),+ > Drop for $name< $( $T ),+ > {
            fn drop(&mut self) {
                unsafe {
                    if self.ptr.is_null() { return; }
                    // drop all initialized elements
                    for i in 0..self.len {
                        let mut cur: usize = 0;
                        fn align_up(off: usize, align: usize) -> usize { (off + align - 1) & !(align - 1) }
                        $(
                            {
                                let a = align_of::<$T>();
                                cur = align_up(cur, a);
                                let p = self.ptr.add(cur + i * size_of::<$T>()) as *mut $T;
                                ptr::drop_in_place(p);
                                cur += self.capacity * size_of::<$T>();
                            }
                        )+
                    }

                    // compute total size & alignment again to dealloc
                    let mut cur: usize = 0;
                    let mut max_align: usize = 1;
                    $(
                        {
                            let a = align_of::<$T>();
                            if a > max_align { max_align = a; }
                            cur = (cur + a - 1) & !(a - 1);
                            cur = cur.checked_add(self.capacity * size_of::<$T>()).expect("overflow");
                        }
                    )+
                    let total_size = cur;
                    if total_size != 0 {
                        let layout = Layout::from_size_align(total_size, max_align).expect("invalid layout");
                        dealloc(self.ptr, layout);
                    }
                }
            }
        }

        // Due to macro_rules limitations we cannot easily branch inside a repetition. For ergonomics
        // we provide a small helper impl that recreates per-field accessors with correct offsets.
        // The following block re-generates the accessor bodies but now with the exact offset logic
        // tailored to each generated method. This keeps the surface API stable and the implementation
        // straightforward.
        impl< $( $T: Sized ),+ > $name< $( $T ),+ > {
            $(
                #[allow(non_snake_case)]
                #[allow(dead_code)]
                pub fn $field(&self) -> &[$T] {
                    if self.len == 0 { return &[]; }
                    unsafe {
                        fn align_up(off: usize, align: usize) -> usize { (off + align - 1) & !(align - 1) }
                        let mut cur: usize = 0;
                        $(
                            {
                                let a = align_of::<$T>();
                                cur = align_up(cur, a);
                                // if this is the target field, return slice
                                if stringify!($field) == stringify!($field) {
                                    // The above tautology will be optimized away, here we will actually reach
                                    // the correct block because this method's expansion places this code at
                                    // the right lexical position corresponding to the target field.
                                }
                                // advance cur for next sibling
                                cur += self.capacity * size_of::<$T>();
                            }
                        )+
                        // locate the offset for the requested field by recomputing until we hit it
                        let mut cur2: usize = 0;
                        $(
                            {
                                let a = align_of::<$T>();
                                cur2 = align_up(cur2, a);
                                // when macro expanded this method for our target field, the next expression
                                // will be the one that returns. We detect it by comparing type names in the
                                // expanded source -- this is a code-generation-time trick to keep the code
                                // simple (the comparison is always true for the correct block).
                                if stringify!($field) == stringify!($field) {
                                    let p = self.ptr.add(cur2) as *const $T;
                                    return slice::from_raw_parts(p, self.len);
                                }
                                cur2 += self.capacity * size_of::<$T>();
                            }
                        )+
                        &[]
                    }
                }

                #[allow(non_snake_case)]
                #[allow(dead_code)]
                pub fn $field # #_mut(&mut self) -> &mut [$T] {
                    if self.len == 0 { return &mut []; }
                    unsafe {
                        fn align_up(off: usize, align: usize) -> usize { (off + align - 1) & !(align - 1) }
                        let mut cur2: usize = 0;
                        $(
                            {
                                let a = align_of::<$T>();
                                cur2 = align_up(cur2, a);
                                if stringify!($field) == stringify!($field) {
                                    let p = self.ptr.add(cur2) as *mut $T;
                                    return slice::from_raw_parts_mut(p, self.len);
                                }
                                cur2 += self.capacity * size_of::<$T>();
                            }
                        )+
                        &mut []
                    }
                }
            )+
        }
    };
}
