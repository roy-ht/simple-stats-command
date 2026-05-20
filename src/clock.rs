use std::time::{SystemTime, UNIX_EPOCH};

/// Local time (HH:MM:SS) at the moment of collection.
pub fn local_hms() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;

    let tm = localtime(secs);
    format!("{:02}:{:02}:{:02}", tm.tm_hour, tm.tm_min, tm.tm_sec)
}

// POSIX `struct tm`. The trailing tm_gmtoff/tm_zone fields exist on both
// macOS and glibc; we include them so localtime_r writes within bounds.
#[repr(C)]
struct Tm {
    tm_sec: i32,
    tm_min: i32,
    tm_hour: i32,
    tm_mday: i32,
    tm_mon: i32,
    tm_year: i32,
    tm_wday: i32,
    tm_yday: i32,
    tm_isdst: i32,
    tm_gmtoff: i64,
    tm_zone: *const i8,
}

fn localtime(secs: i64) -> Tm {
    unsafe extern "C" {
        fn tzset();
        fn localtime_r(timep: *const i64, result: *mut Tm) -> *mut Tm;
    }

    // glibc's localtime_r does not call tzset() itself; call it so the TZ
    // environment / system zone is honored.
    let mut tm: Tm = unsafe { std::mem::zeroed() };
    unsafe {
        tzset();
        localtime_r(&secs, &mut tm);
    }
    tm
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_local_hms_format() {
        let s = local_hms();
        assert_eq!(s.len(), 8);
        let b = s.as_bytes();
        assert_eq!(b[2], b':');
        assert_eq!(b[5], b':');
        assert!(s.chars().enumerate().all(|(i, c)| {
            if i == 2 || i == 5 { c == ':' } else { c.is_ascii_digit() }
        }));
    }
}
