#[cfg(test)]
mod macro_test {
    use std::io::{Error as IoError, ErrorKind};
    use std::net::AddrParseError;
    use erret_macro::Error;

    #[derive(Debug, Error)]
    pub enum TestError {
        Zalupa,

        #[error("Empty err")]
        Empty,

        #[error("Iface '{}' not found")]
        InvalidIface(&'static str),

        #[error("Bad port: {}")]
        BadPort(u16),
        
        #[error("Disk err: {}")]
        #[from]
        Io(IoError),
        
        #[error("Bad IP: {}")]
        #[from]
        BadIp(AddrParseError),

        #[error("Zaebal, suka, blyat: {}")]
        ZaebalIo(IoError),
    }

    type Result<T> = std::result::Result<T, TestError>;

    #[test]
    fn test_display_formatting() {
        let err_zalupa = TestError::Zalupa;
        assert_eq!(err_zalupa.to_string(), "Zalupa");

        let err_empty = TestError::Empty;
        assert_eq!(err_empty.to_string(), "Empty err");

        let err_port = TestError::BadPort(80);
        assert_eq!(err_port.to_string(), "Bad port: 80");

        let err_iface = TestError::InvalidIface("eth0");
        assert_eq!(err_iface.to_string(), "Iface 'eth0' not found");

        let io_sys = IoError::new(ErrorKind::PermissionDenied, "access denied");
        let err_io = TestError::Io(io_sys);
        assert_eq!(err_io.to_string(), "Disk err: access denied");

        let zaebal_sys = IoError::new(ErrorKind::PermissionDenied, "access denied");
        let err_zaebalio = TestError::ZaebalIo(zaebal_sys);
        assert_eq!(err_zaebalio.to_string(), "Zaebal, suka, blyat: access denied");
    }

    fn cause_io_error() -> std::result::Result<(), IoError> {
        Err(IoError::new(ErrorKind::NotFound, "file not found"))
    }

    fn cause_ip_error() -> std::result::Result<(), AddrParseError> {
        "256.256.256.256".parse::<std::net::Ipv4Addr>().map(|_| ())
    }

    fn low_level_logic(trigger: u8) -> Result<()> {
        if trigger == 1 {
            cause_io_error()?;
        } else if trigger == 2 {
            cause_ip_error()?;
        }
        Ok(())
    }

    #[test]
    fn test_question_mark_operator() {
        let res_io = low_level_logic(1);
        assert!(res_io.is_err());
        match res_io.unwrap_err() {
            TestError::Io(e) => assert_eq!(e.kind(), ErrorKind::NotFound),
            _ => panic!("Expected TestError::Io"),
        }

        let res_ip = low_level_logic(2);
        assert!(res_ip.is_err());
        assert!(matches!(res_ip.unwrap_err(), TestError::BadIp(_)));
    }

    #[test]
    fn test_anyhow_compatibility() {
        fn high_level_workflow() -> anyhow::Result<()> {
            low_level_logic(1)?; 
            Ok(())
        }

        let res = high_level_workflow();
        assert!(res.is_err());
        
        let anyhow_err = res.unwrap_err();
        assert_eq!(anyhow_err.to_string(), "Disk err: file not found");
    }

    #[test]
    fn test_zero_cost_size() {
        let size = std::mem::size_of::<TestError>();
        println!("Stack size: {} byte", size);
        assert!(size <= 24, "Err too large");
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