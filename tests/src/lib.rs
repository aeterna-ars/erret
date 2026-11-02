#[allow(dead_code)]

#[cfg(test)]
mod macro_test {
    use core::fmt::Debug;
    use std::error::Error as StdError;
    use std::io::{Error as IoError, ErrorKind};
    use std::net::AddrParseError;

    use erret_macro::Error;

    #[derive(Debug, Error)]
    pub enum ShapeErr {
        Bare,

        #[error("explicit message")]
        Message,

        #[error("port={}")]
        Port(u16),

        #[error("a={} b={}")]
        Pair(u32, u32),

        #[error("name: {name}")]
        Named { name: &'static str },

        #[error("x={x} y={y}")]
        Point { x: i32, y: i32 },
    }

    #[test]
    fn display_all_shapes() {
        assert_eq!(ShapeErr::Bare.to_string(), "Bare");
        assert_eq!(ShapeErr::Message.to_string(), "explicit message");
        assert_eq!(ShapeErr::Port(8080).to_string(), "port=8080");
        assert_eq!(ShapeErr::Pair(1, 2).to_string(), "a=1 b=2");
        assert_eq!(
            ShapeErr::Named { name: "abc" }.to_string(),
            "name: abc"
        );
        assert_eq!(
            ShapeErr::Point { x: -5, y: 10 }.to_string(),
            "x=-5 y=10"
        );
    }

    #[derive(Debug, Error)]
    pub enum FmtErr {
        #[error("{0:?}")]
        Debug0(String),

        #[error("{0} vs {1}")]
        Ordered(&'static str, &'static str),

        #[error("{1} then {0}")]
        Swapped(&'static str, &'static str),

        #[error("{name} = {name}")]
        Repeated { name: &'static str },

        #[error("{0:#x}")]
        Hex(u32),
    }

    #[test]
    fn display_format_specs() {
        assert_eq!(FmtErr::Debug0("hi".into()).to_string(), "\"hi\"");
        assert_eq!(FmtErr::Ordered("a", "b").to_string(), "a vs b");
        assert_eq!(FmtErr::Swapped("a", "b").to_string(), "b then a");
        assert_eq!(FmtErr::Repeated { name: "q" }.to_string(), "q = q");
        assert_eq!(FmtErr::Hex(255).to_string(), "0xff");
    }

    #[derive(Debug, Error)]
    pub enum NoPlaceholder {
        #[error("nothing to see")]
        WithField(u64),

        #[error("also nothing")]
        WithNamed { _x: u8 },
    }

    #[test]
    fn display_no_placeholder_with_fields() {
        assert_eq!(NoPlaceholder::WithField(999).to_string(), "nothing to see");
        assert_eq!(
            NoPlaceholder::WithNamed { _x: 1 }.to_string(),
            "also nothing"
        );
    }

    #[derive(Debug, Error)]
    pub enum FromTuple {
        #[error("io: {}")]
        #[from]
        Io(IoError),

        #[error("ip: {}")]
        #[from]
        Ip(AddrParseError),
    }

    #[test]
    fn from_tuple_impls() {
        let e: FromTuple = IoError::new(ErrorKind::Other, "x").into();
        assert!(matches!(e, FromTuple::Io(_)));

        let bad: AddrParseError = "nope".parse::<std::net::Ipv4Addr>().unwrap_err();
        let e: FromTuple = bad.into();
        assert!(matches!(e, FromTuple::Ip(_)));
    }

    #[derive(Debug, Error)]
    pub enum FromNamedField {
        #[error("io in {label}")]
        Io {
            label: &'static str,
            #[from]
            source: IoError,
        },
    }

    #[test]
    fn from_named_field() {
        let e: FromNamedField = IoError::new(ErrorKind::Other, "x").into();
        match &e {
            FromNamedField::Io { label, .. } => assert_eq!(*label, "Io"),
        }
        assert_eq!(e.to_string(), "io in Io");
    }

    type R = std::result::Result<(), FromTuple>;

    fn depth_io() -> std::result::Result<(), IoError> {
        Err(IoError::new(ErrorKind::PermissionDenied, "denied"))
    }

    fn depth_ip() -> std::result::Result<(), AddrParseError> {
        "256.0.0.0".parse::<std::net::Ipv4Addr>().map(|_| ())
    }

    fn layer(sel: u8) -> R {
        match sel {
            1 => depth_io()?,
            2 => depth_ip()?,
            _ => {}
        }
        Ok(())
    }

    #[test]
    fn question_mark_chain() {
        let e = layer(1).unwrap_err();
        match e {
            FromTuple::Io(io) => assert_eq!(io.kind(), ErrorKind::PermissionDenied),
            _ => panic!("expected Io"),
        }

        let e = layer(2).unwrap_err();
        assert!(matches!(e, FromTuple::Ip(_)));

        assert!(layer(0).is_ok());
    }

    #[derive(Debug, Error)]
    pub enum GenErr<T>
    where
        T: Debug,
    {
        #[error("generic: {0:?}")]
        One(T),

        #[error("both: {0:?} and {1:?}")]
        Two(T, T),
    }

    #[test]
    fn generic_display() {
        let e: GenErr<u32> = GenErr::One(42);
        assert_eq!(e.to_string(), "generic: 42");

        let e: GenErr<&str> = GenErr::Two("a", "b");
        assert_eq!(e.to_string(), "both: \"a\" and \"b\"");
    }

    #[derive(Debug, Error)]
    pub enum MultiGen<A, B>
    where
        A: Debug,
        B: Debug,
    {
        #[error("a={0:?}")]
        A(A),
        #[error("b={0:?}")]
        B(B),
    }

    #[test]
    fn multi_generic_display() {
        let e: MultiGen<u8, &str> = MultiGen::A(1);
        assert_eq!(e.to_string(), "a=1");
        let e: MultiGen<u8, &str> = MultiGen::B("x");
        assert_eq!(e.to_string(), "b=\"x\"");
    }

    #[derive(Debug, Error)]
    pub enum ConstErr<const N: usize> {
        #[error("capacity {N}")]
        Cap,
    }

    #[test]
    fn const_generic_display() {
        let e: ConstErr<8> = ConstErr::Cap;
        assert_eq!(e.to_string(), "capacity 8");
    }

    #[test]
    fn traits_are_implemented() {
        fn assert_display<T: std::fmt::Display>() {}
        fn assert_debug<T: Debug>() {}
        fn assert_error<T: StdError>() {}

        assert_display::<ShapeErr>();
        assert_debug::<ShapeErr>();
        assert_error::<ShapeErr>();
    }

    #[test]
    fn source_is_none() {
        assert!(ShapeErr::Bare.source().is_none());
        assert!(ShapeErr::Port(1).source().is_none());
    }

    #[test]
    fn boxed_dyn_error() {
        let e: Box<dyn StdError + Send + Sync + 'static> =
            Box::new(ShapeErr::Message);
        assert_eq!(e.to_string(), "explicit message");
    }

    #[test]
    fn anyhow_compat() {
        fn workflow() -> anyhow::Result<()> {
            depth_io().map_err(FromTuple::from)?;
            Ok(())
        }
        let err = workflow().unwrap_err();
        assert_eq!(err.to_string(), "io: denied");
    }

    #[test]
    fn nested_in_format() {
        let s = format!("[{}]", ShapeErr::Port(80));
        assert_eq!(s, "[port=80]");
    }

    #[derive(Debug, Error, Clone, Copy, PartialEq, Eq)]
    pub enum CopyErr {
        #[error("a")]
        A,
        #[error("b")]
        B,
    }

    #[test]
    fn user_derives_survive() {
        let a = CopyErr::A;
        let b = a;
        assert_eq!(a, b);
        assert_ne!(a, CopyErr::B);
    }

    #[derive(Debug, Error)]
    pub enum ManyVariants {
        #[error("v0")]  V0,
        #[error("v1")]  V1,
        #[error("v2")]  V2,
        #[error("v3")]  V3,
        #[error("v4")]  V4,
        #[error("v5")]  V5,
        #[error("v6")]  V6,
        #[error("v7")]  V7,
        #[error("v8")]  V8,
        #[error("v9")]  V9,
    }

    #[test]
    fn many_variants_compile_and_display() {
        assert_eq!(ManyVariants::V0.to_string(), "v0");
        assert_eq!(ManyVariants::V9.to_string(), "v9");
    }

    #[test]
    fn size_is_reasonable() {
        let sz = std::mem::size_of::<ShapeErr>();
        println!("ShapeErr size: {sz}");
        assert!(sz <= 32, "ShapeErr too large: {sz}");
    }

    #[derive(Debug, Error)]
    pub enum Layered {
        #[error("io: {}")]
        #[from]
        Io(IoError),

        #[error("ip: {}")]
        #[from]
        Ip(AddrParseError),
    }

    fn inner(sel: u8) -> std::result::Result<(), IoError> {
        if sel == 0 {
            Ok(())
        } else {
            Err(IoError::new(ErrorKind::Other, "boom"))
        }
    }

    fn mid(sel: u8) -> std::result::Result<(), Layered> {
        inner(sel)?;
        Ok(())
    }

    #[test]
    fn layered_question_mark() {
        assert!(mid(0).is_ok());
        let e = mid(1).unwrap_err();
        assert_eq!(e.to_string(), "io: boom");
    }
}

#[cfg(test)]
mod result_test {
    use erret_result::*;
    use std::fs::File;
    use std::net::IpAddr;
    use std::fmt;

    fn parse_number(s: &str) -> ErrResult<u32> {
        let num = s.parse::<u32>()?;
        Ok(num)
    }

    fn open_broken_file() -> ErrResult<File> {
        let file = File::open("non_existent_file_123.txt")?;
        Ok(file)
    }

    fn parse_ip(s: &str) -> ErrResult<IpAddr> {
        let ip = s.parse::<IpAddr>()?;
        Ok(ip)
    }

    #[test]
    fn test_all_standard_errors() {
        let res_num = parse_number("nope");
        assert!(res_num.is_err());
        println!("Display parse: {}", res_num.unwrap_err());

        let res_file = open_broken_file();
        assert!(res_file.is_err());
        println!("Display file: {}", res_file.unwrap_err());

        let res_ip = parse_ip("300.400.500.600");
        assert!(res_ip.is_err());
        println!("Display net: {}", res_ip.unwrap_err());
    }

    #[derive(Debug)]
    struct CustomError(String);

    impl fmt::Display for CustomError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "custom error: {}", self.0)
        }
    }

    impl std::error::Error for CustomError {}

    #[test]
    fn question_mark_converts_std_error() {
        fn read() -> Result<String, ErrRet> {
            let s = std::fs::read_to_string("/nonexistent/path/foo.txt")?;
            Ok(s)
        }

        let err = read().unwrap_err();
        assert!(!err.to_string().is_empty());
    }

    #[test]
    fn question_mark_converts_custom_error() {
        fn fail() -> Result<(), ErrRet> {
            fn inner() -> std::result::Result<(), CustomError> {
                Err(CustomError("boom".into()))
            }
            inner()?;
            Ok(())
        }

        let err = fail().unwrap_err();
        assert_eq!(err.to_string(), "custom error: boom");
    }

    #[test]
    fn question_mark_short_circuits_on_ok() {
        fn ok_path() -> Result<i32, ErrRet> {
            fn inner() -> std::result::Result<i32, CustomError> {
                Ok(42)
            }
            let v = inner()?;
            Ok(v)
        }

        assert_eq!(ok_path().unwrap(), 42);
    }
}