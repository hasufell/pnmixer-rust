#[macro_export]
macro_rules! create_builder_item {
    ($sname:ident, $($element: ident: $ty: ty),+) => {
        pub struct $sname {
            $(
               pub $element: $ty
             ),+
        }

        impl $sname {
            pub fn new(builder: gtk::Builder) -> $sname {
                return $sname {
                    $(
                        $element: builder.get_object(stringify!($element)).unwrap()
                    ),+
                };
            }
        }
    }
}

