#![allow(unused)]
#![feature(macro_metavar_expr)]

#[cfg(test)]
mod tests {
    use sibling_vecs::*;

    #[test]
    fn compiles_and_new() {
        sibling_vecs!(
            pub struct SiblingVecs3 {
                a: i32,
                b: f64,
                c: bool,
            }
        );
        let _v = SiblingVecs3::new();
    }

    #[test]
    fn push_and_read() {
        sibling_vecs!(
            pub struct SiblingVecs3 {
                a: i32,
                b: f64,
                c: bool,
            }
        );

        let mut v = SiblingVecs3::new();
        v.push(1, 2.0, true);
        v.push(10, 20.0, false);

        let (a, b, c) = v.as_slices();
        assert_eq!(a, &[1, 10]);
        assert_eq!(b, &[2.0, 20.0]);
        assert_eq!(c, &[true, false]);
    }

    #[test]
    fn mut_slices() {
        sibling_vecs!(
            pub struct SiblingVecs3 {
                a: i32,
                b: f64,
                c: bool,
            }
        );

        let mut v = SiblingVecs3::new();
        v.push(5, 3.5, false);
        {
            let (a, b, c) = v.as_mut_slices();
            a[0] = 42;
            b[0] *= 2.0;
            c[0] = true;
        }
        let (a, b, c) = v.as_slices();
        assert_eq!(a[0], 42);
        assert_eq!(b[0], 7.0);
        assert!(c[0]);
    }

    #[test]
    fn resize_triggers_realloc() {
        sibling_vecs!(
            pub struct SiblingVecs3 {
                a: i32,
                b: f64,
                c: bool,
            }
        );

        let mut v = SiblingVecs3::with_capacity(2);
        for i in 0..10 {
            v.push(i as i32, i as f64, i % 2 == 0);
        }
        let (a, b, c) = v.as_slices();
        assert_eq!(a.len(), 10);
        assert_eq!(a[9], 9);
        assert_eq!(b[9], 9.0);
        assert!(!c[9]); // 9 % 2 != 0
    }

    #[test]
    fn drop_works_no_panic() {
        sibling_vecs!(
            pub struct SiblingVecs3 {
                a: i32,
                b: f64,
                c: bool,
            }
        );

        let mut v = SiblingVecs3::new();
        v.push(1, 1.0, true);
        v.push(2, 2.0, false);
        // just drop
        drop(v);
    }

    #[test]
    fn zero_capacity_edge() {
        sibling_vecs!(
            pub struct SiblingVecs2 {
                a: i32,
                b: f64,
            }
        );
        let v = SiblingVecs2::new();
        assert!(v.ptr.is_null());
        assert_eq!(v.cap, 0);
        assert_eq!(v.len, 0);
    }

    #[test]
    fn alignment_and_padding_handled() {
        sibling_vecs!(
            pub struct SiblingVecsU {
                a: u8,
                b: u64,
            }
        );
        let mut v = SiblingVecsU::new();
        for i in 0..5 {
            v.push(i, i as u64 * 1000);
        }
        let (small, big) = v.as_slices();
        assert_eq!(small, &[0, 1, 2, 3, 4]);
        assert_eq!(big, &[0, 1000, 2000, 3000, 4000]);
    }
}
