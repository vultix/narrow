#[cfg(feature = "derive")]
mod tests {
    mod derive {
        mod r#struct {
            mod unit {
                use narrow::{
                    array::{StructArray, VariableSizeListArray},
                    bitmap::ValidityBitmap,
                    buffer::BoxBuffer,
                    ArrayType, Length,
                };

                #[derive(ArrayType, Copy, Clone, Default)]
                struct Foo;

                #[derive(ArrayType, Copy, Clone, Default)]
                struct Bar<const N: bool = false>
                where
                    Self: Sized;

                #[test]
                fn non_nullable() {
                    let input = [Foo; 5];
                    let array = input.into_iter().collect::<StructArray<Foo>>();
                    assert_eq!(array.len(), 5);
                }

                #[test]
                fn nullable() {
                    let input = [Some(Foo); 5];
                    let array = input.into_iter().collect::<StructArray<Foo, true>>();
                    assert_eq!(array.len(), 5);
                    assert!(array.all_valid());
                }

                #[test]
                fn generic() {
                    let input = [Bar, Bar];
                    let array = input.into_iter().collect::<StructArray<Bar>>();
                    assert_eq!(array.len(), 2);
                }

                #[test]
                fn nested() {
                    let input = vec![
                        Some(vec![Foo; 1]),
                        None,
                        Some(vec![Foo; 2]),
                        Some(vec![Foo; 3]),
                    ];
                    let array = input
                        .into_iter()
                        .collect::<VariableSizeListArray<StructArray<Foo>, true>>();
                    assert_eq!(array.len(), 4);
                }

                #[test]
                fn buffer() {
                    let input = [Foo; 5];
                    let array = input
                        .into_iter()
                        .collect::<StructArray<Foo, false, BoxBuffer>>();
                    assert_eq!(array.len(), 5);
                }
            }

            mod unnamed {
                use narrow::{
                    array::{StructArray, VariableSizeListArray},
                    bitmap::ValidityBitmap,
                    ArrayType, Length,
                };

                #[derive(ArrayType, Default)]
                struct Foo<'a>(pub u32, pub u16, &'a str);

                #[derive(ArrayType, Default)]
                struct Bar<'a>(Foo<'a>);

                #[derive(ArrayType, Default)]
                struct FooBar<'a, T>(Bar<'a>, T);

                #[test]
                fn non_nullable() {
                    let input = [Foo(1, 2, "as"), Foo(3, 4, "df")];
                    let array = input.into_iter().collect::<StructArray<Foo>>();
                    assert_eq!(array.len(), 2);
                    assert_eq!(array.0 .0 .0, &[1, 3]);
                    assert_eq!(array.0 .1 .0, &[2, 4]);
                    assert_eq!(
                        array.0 .2 .0 .0 .0.data.0.as_slice(),
                        &[b'a', b's', b'd', b'f']
                    );
                    assert_eq!(array.0 .2 .0 .0 .0.offsets.as_slice(), &[0, 2, 4]);

                    let input = [
                        Bar(Foo(1, 2, "hello")),
                        Bar(Foo(3, 4, "world")),
                        Bar(Foo(5, 6, "!")),
                    ];
                    let array = input.into_iter().collect::<StructArray<Bar>>();
                    assert_eq!(array.len(), 3);
                }

                #[test]
                fn nullable() {
                    let input = [Some(Foo(1, 2, "n")), None, Some(Foo(3, 4, "arrow"))];
                    let array = input.into_iter().collect::<StructArray<Foo, true>>();
                    assert_eq!(array.len(), 3);
                    assert_eq!(array.is_valid(0), Some(true));
                    assert_eq!(array.is_null(1), Some(true));
                    assert_eq!(array.is_valid(2), Some(true));

                    let input = [Some(Bar(Foo(1, 2, "yes"))), None];
                    let array = input.into_iter().collect::<StructArray<Bar, true>>();
                    assert_eq!(array.len(), 2);
                }

                #[test]
                fn generic() {
                    let input = [
                        FooBar(Bar(Foo(1, 2, "n")), false),
                        FooBar(Bar(Foo(1, 2, "arrow")), false),
                    ];
                    let array = input.into_iter().collect::<StructArray<FooBar<_>>>();
                    assert_eq!(array.len(), 2);
                }

                #[test]
                fn nested() {
                    let input = vec![
                        Some(vec![Some(FooBar(Bar(Foo(42, 0, "!")), 1234))]),
                        None,
                        Some(vec![None]),
                        Some(vec![None, None]),
                    ];
                    let array = input
                        .into_iter()
                        .collect::<VariableSizeListArray<StructArray<FooBar<_>, true>, true>>();
                    assert_eq!(array.len(), 4);
                }
            }

            mod named {
                use narrow::{
                    array::{StructArray, VariableSizeListArray},
                    bitmap::ValidityBitmap,
                    ArrayType, Length,
                };

                #[derive(ArrayType)]
                struct Foo<'a, T: ?Sized> {
                    a: &'a T,
                    b: bool,
                    c: u8,
                }
                impl<'a, T: ?Sized> Default for Foo<'a, T>
                where
                    &'a T: Default,
                {
                    fn default() -> Self {
                        Self {
                            a: Default::default(),
                            b: Default::default(),
                            c: Default::default(),
                        }
                    }
                }

                #[derive(ArrayType, Default)]
                struct Bar<T> {
                    a: u32,
                    b: Option<bool>,
                    c: T,
                }

                #[derive(ArrayType, Default)]
                struct FooBar {
                    foo: bool,
                    bar: Bar<()>,
                }

                #[test]
                fn non_nullable() {
                    let input = [
                        Foo {
                            a: "as",
                            b: true,
                            c: 4,
                        },
                        Foo {
                            a: "df",
                            b: false,
                            c: 2,
                        },
                    ];
                    let array = input.into_iter().collect::<StructArray<Foo<_>>>();
                    assert_eq!(array.len(), 2);
                    assert_eq!(array.0.c.0, &[4, 2]);
                }

                #[test]
                fn nullable() {
                    let input = [
                        Some(Bar {
                            a: 1,
                            b: Some(false),
                            c: None,
                        }),
                        None,
                        Some(Bar {
                            a: 2,
                            b: None,
                            c: Some(()),
                        }),
                    ];
                    let array = input.into_iter().collect::<StructArray<Bar<_>, true>>();
                    assert_eq!(array.len(), 3);
                    assert_eq!(array.is_valid(0), Some(true));
                    assert_eq!(array.is_null(1), Some(true));
                    assert_eq!(array.is_valid(2), Some(true));

                    let input = [
                        Some(Bar {
                            a: 1,
                            b: None,
                            c: false,
                        }),
                        None,
                    ];
                    let array = input.into_iter().collect::<StructArray<Bar<_>, true>>();
                    assert_eq!(array.len(), 2);
                }

                #[test]
                fn generic() {
                    let input = [
                        Some(Bar {
                            a: 1,
                            b: Some(false),
                            c: Foo {
                                a: "a",
                                b: false,
                                c: 123,
                            },
                        }),
                        None,
                    ];
                    let array = input
                        .into_iter()
                        .collect::<StructArray<Bar<Foo<str>>, true>>();
                    assert_eq!(array.len(), 2);
                }

                #[test]
                fn nested() {
                    let input = vec![
                        Some(vec![Some(Bar {
                            a: 2,
                            b: None,
                            c: Foo {
                                a: "a",
                                b: false,
                                c: 123,
                            },
                        })]),
                        None,
                        Some(vec![None]),
                        Some(vec![None, None]),
                    ];
                    let array = input
                        .into_iter()
                        .collect::<VariableSizeListArray<StructArray<Bar<Foo<str>>, true>, true>>();
                    assert_eq!(array.len(), 4);
                }
            }
        }
    }
}
