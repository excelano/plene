fn long_function() {
    let v0 = 0;
    let v1 = 1;
    let v2 = 2;
    let v3 = 3;
    let v4 = 4;
    let v5 = 5;
    let v6 = 6;
    let v7 = 7;
    let v8 = 8;
    let v9 = 9;
    let v10 = 10;
    let v11 = 11;
    let v12 = 12;
    let v13 = 13;
    let v14 = 14;
    let v15 = 15;
    let v16 = 16;
    let v17 = 17;
    let v18 = 18;
    let v19 = 19;
}

fn short_function() {
    let v0 = 0;
    let v1 = 1;
}

fn just_under() {
    let v0 = 0;
    let v1 = 1;
    let v2 = 2;
    let v3 = 3;
    let v4 = 4;
    let v5 = 5;
    let v6 = 6;
    let v7 = 7;
    let v8 = 8;
    let v9 = 9;
    let v10 = 10;
    let v11 = 11;
    let v12 = 12;
    let v13 = 13;
    let v14 = 14;
    let v15 = 15;
    let v16 = 16;
}

fn just_over() {
    let v0 = 0;
    let v1 = 1;
    let v2 = 2;
    let v3 = 3;
    let v4 = 4;
    let v5 = 5;
    let v6 = 6;
    let v7 = 7;
    let v8 = 8;
    let v9 = 9;
    let v10 = 10;
    let v11 = 11;
    let v12 = 12;
    let v13 = 13;
    let v14 = 14;
    let v15 = 15;
    let v16 = 16;
    let v17 = 17;
}

struct Wide {
    pub f0: u8,
    pub f1: u8,
    pub f2: u8,
    pub f3: u8,
    pub f4: u8,
    pub f5: u8,
    pub f6: u8,
    pub f7: u8,
    pub f8: u8,
    pub f9: u8,
    pub f10: u8,
    pub f11: u8,
    pub f12: u8,
    pub f13: u8,
    pub f14: u8,
    pub f15: u8,
    pub f16: u8,
    pub f17: u8,
    pub f18: u8,
    pub f19: u8,
}

enum Big {
    V0,
    V1,
    V2,
    V3,
    V4,
    V5,
    V6,
    V7,
    V8,
    V9,
    V10,
    V11,
    V12,
    V13,
    V14,
    V15,
    V16,
    V17,
    V18,
    V19,
}

trait Wordy {
    fn m0(&self);
    fn m1(&self);
    fn m2(&self);
    fn m3(&self);
    fn m4(&self);
    fn m5(&self);
    fn m6(&self);
    fn m7(&self);
    fn m8(&self);
    fn m9(&self);
    fn m10(&self);
    fn m11(&self);
    fn m12(&self);
    fn m13(&self);
    fn m14(&self);
    fn m15(&self);
    fn m16(&self);
    fn m17(&self);
    fn m18(&self);
    fn m19(&self);
}

mod outer {
    let v0 = 0;
    mod inner {
        fn f0() {}
        fn f1() {}
        fn f2() {}
        fn f3() {}
        fn f4() {}
        fn f5() {}
        fn f6() {}
        fn f7() {}
        fn f8() {}
        fn f9() {}
        fn f10() {}
        fn f11() {}
        fn f12() {}
        fn f13() {}
        fn f14() {}
        fn f15() {}
        fn f16() {}
        fn f17() {}
        fn f18() {}
        fn f19() {}
    }
}

impl Wide {
    fn a0(&self) {}
    fn a1(&self) {}
    fn a2(&self) {}
    fn a3(&self) {}
    fn a4(&self) {}
    fn a5(&self) {}
    fn a6(&self) {}
    fn a7(&self) {}
    fn a8(&self) {}
    fn a9(&self) {}
    fn a10(&self) {}
    fn a11(&self) {}
    fn a12(&self) {}
    fn a13(&self) {}
    fn a14(&self) {}
    fn a15(&self) {}
    fn a16(&self) {}
    fn a17(&self) {}
    fn a18(&self) {}
    fn a19(&self) {}
}

impl<T: Clone> Wordy for Vec<T> {
    fn m0(&self) {}
    fn m1(&self) {}
    fn m2(&self) {}
    fn m3(&self) {}
    fn m4(&self) {}
    fn m5(&self) {}
    fn m6(&self) {}
    fn m7(&self) {}
    fn m8(&self) {}
    fn m9(&self) {}
    fn m10(&self) {}
    fn m11(&self) {}
    fn m12(&self) {}
    fn m13(&self) {}
    fn m14(&self) {}
    fn m15(&self) {}
    fn m16(&self) {}
    fn m17(&self) {}
    fn m18(&self) {}
    fn m19(&self) {}
}

impl fmt::Display for module::Wide {
    fn d0(&self) {}
    fn d1(&self) {}
    fn d2(&self) {}
    fn d3(&self) {}
    fn d4(&self) {}
    fn d5(&self) {}
    fn d6(&self) {}
    fn d7(&self) {}
    fn d8(&self) {}
    fn d9(&self) {}
    fn d10(&self) {}
    fn d11(&self) {}
    fn d12(&self) {}
    fn d13(&self) {}
    fn d14(&self) {}
    fn d15(&self) {}
    fn d16(&self) {}
    fn d17(&self) {}
    fn d18(&self) {}
    fn d19(&self) {}
}

impl Wordy for &Wide {
    fn r0(&self) {}
    fn r1(&self) {}
    fn r2(&self) {}
    fn r3(&self) {}
    fn r4(&self) {}
    fn r5(&self) {}
    fn r6(&self) {}
    fn r7(&self) {}
    fn r8(&self) {}
    fn r9(&self) {}
    fn r10(&self) {}
    fn r11(&self) {}
    fn r12(&self) {}
    fn r13(&self) {}
    fn r14(&self) {}
    fn r15(&self) {}
    fn r16(&self) {}
    fn r17(&self) {}
    fn r18(&self) {}
    fn r19(&self) {}
}

impl Wordy for [u8] {
    fn s0(&self) {}
    fn s1(&self) {}
    fn s2(&self) {}
    fn s3(&self) {}
    fn s4(&self) {}
    fn s5(&self) {}
    fn s6(&self) {}
    fn s7(&self) {}
    fn s8(&self) {}
    fn s9(&self) {}
    fn s10(&self) {}
    fn s11(&self) {}
    fn s12(&self) {}
    fn s13(&self) {}
    fn s14(&self) {}
    fn s15(&self) {}
    fn s16(&self) {}
    fn s17(&self) {}
    fn s18(&self) {}
    fn s19(&self) {}
}

fn nested_blocks() {
    if true {
        let v0 = 0;
        let v1 = 1;
        let v2 = 2;
        let v3 = 3;
        let v4 = 4;
        let v5 = 5;
        let v6 = 6;
        let v7 = 7;
        let v8 = 8;
        let v9 = 9;
        let v10 = 10;
        let v11 = 11;
        let v12 = 12;
        let v13 = 13;
        let v14 = 14;
        let v15 = 15;
        let v16 = 16;
        let v17 = 17;
        let v18 = 18;
        let v19 = 19;
    }
    let f = || {
        let v0 = 0;
        let v1 = 1;
        let v2 = 2;
        let v3 = 3;
        let v4 = 4;
        let v5 = 5;
        let v6 = 6;
        let v7 = 7;
        let v8 = 8;
        let v9 = 9;
        let v10 = 10;
        let v11 = 11;
        let v12 = 12;
        let v13 = 13;
        let v14 = 14;
        let v15 = 15;
        let v16 = 16;
        let v17 = 17;
        let v18 = 18;
        let v19 = 19;
    };
    loop {
        let v0 = 0;
        let v1 = 1;
        let v2 = 2;
        let v3 = 3;
        let v4 = 4;
        let v5 = 5;
        let v6 = 6;
        let v7 = 7;
        let v8 = 8;
        let v9 = 9;
        let v10 = 10;
        let v11 = 11;
        let v12 = 12;
        let v13 = 13;
        let v14 = 14;
        let v15 = 15;
        let v16 = 16;
        let v17 = 17;
        let v18 = 18;
        let v19 = 19;
    }
}

enum Variants {
    Named {
        pub f0: u8,
        pub f1: u8,
        pub f2: u8,
        pub f3: u8,
        pub f4: u8,
        pub f5: u8,
        pub f6: u8,
        pub f7: u8,
        pub f8: u8,
        pub f9: u8,
        pub f10: u8,
        pub f11: u8,
        pub f12: u8,
        pub f13: u8,
        pub f14: u8,
        pub f15: u8,
        pub f16: u8,
        pub f17: u8,
        pub f18: u8,
        pub f19: u8,
    },
    Other,
}

fn trailing() {
    let v0 = 0;
    let v1 = 1;
    let v2 = 2;
    let v3 = 3;
    let v4 = 4;
    let v5 = 5;
    let v6 = 6;
    let v7 = 7;
    let v8 = 8;
    let v9 = 9;
    let v10 = 10;
    let v11 = 11;
    let v12 = 12;
    let v13 = 13;
    let v14 = 14;
    let v15 = 15;
    let v16 = 16;
    let v17 = 17;
    let v18 = 18;
    let v19 = 19;
} // after
