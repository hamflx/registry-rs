use std::convert::TryInto;

use registry::{Hive, Security};
use utfx::U16CString;
use windows_sys::Win32::{
    Foundation::{HANDLE, LUID},
    Security::{
        AdjustTokenPrivileges, LookupPrivilegeValueW, LUID_AND_ATTRIBUTES, SE_PRIVILEGE_ENABLED,
        TOKEN_ADJUST_PRIVILEGES, TOKEN_PRIVILEGES,
    },
    System::{
        SystemServices::{SE_BACKUP_NAME, SE_RESTORE_NAME},
        Threading::{GetCurrentProcess, OpenProcessToken},
    },
};

fn main() -> Result<(), std::io::Error> {
    let mut token = 0;
    let r = unsafe { OpenProcessToken(GetCurrentProcess(), TOKEN_ADJUST_PRIVILEGES, &mut token) };
    if r == 0 {
        return Err(std::io::Error::last_os_error());
    }

    set_privilege(token, SE_RESTORE_NAME)?;
    set_privilege(token, SE_BACKUP_NAME)?;
    let hive_key = Hive::load_file(
        r"C:\Users\Default\NTUSER.DAT",
        Security::Read | Security::Write,
    )
    .unwrap();

    let keys: Vec<_> = hive_key.keys().map(|k| k.unwrap().to_string()).collect();

    println!("{:?}", keys);
    Ok(())
}

fn set_privilege(handle: HANDLE, name: &str) -> Result<(), std::io::Error> {
    let mut luid: LUID = LUID {
        LowPart: 0,
        HighPart: 0,
    };
    let name: U16CString = name.try_into().unwrap();
    let r = unsafe { LookupPrivilegeValueW(std::ptr::null(), name.as_ptr(), &mut luid) };
    if r == 0 {
        return Err(std::io::Error::last_os_error());
    }

    let mut privilege = TOKEN_PRIVILEGES {
        PrivilegeCount: 1,
        Privileges: [LUID_AND_ATTRIBUTES {
            Luid: luid,
            Attributes: SE_PRIVILEGE_ENABLED,
        }],
    };

    let r = unsafe {
        AdjustTokenPrivileges(
            handle,
            false as i32,
            &mut privilege,
            std::mem::size_of::<TOKEN_PRIVILEGES>() as u32,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        )
    };

    if r == 0 {
        return Err(std::io::Error::last_os_error());
    }
    Ok(())
}
