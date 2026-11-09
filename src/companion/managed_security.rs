//! A protected child DACL alone is insufficient when an unprivileged owner or
//! ancestor can replace the directory. Reject such service locations, including
//! junctions, before registration and on every service start.
use super::{Result, error};
use std::{
    os::windows::{ffi::OsStrExt, fs::MetadataExt},
    path::Path,
};
use windows_sys::Win32::{
    Foundation::LocalFree,
    Security::{Authorization::*, *},
    Storage::FileSystem::{DELETE, WRITE_DAC, WRITE_OWNER},
};

fn trusted(sid: PSID) -> Result<bool> {
    let mut string = std::ptr::null_mut();
    if unsafe { ConvertSidToStringSidW(sid, &mut string) } == 0 {
        return Err(std::io::Error::last_os_error().into());
    }
    let mut count = 0;
    unsafe {
        while *string.add(count) != 0 {
            count += 1;
        }
    }
    let value = unsafe { String::from_utf16_lossy(std::slice::from_raw_parts(string, count)) };
    unsafe {
        LocalFree(string.cast());
    }
    Ok(matches!(
        value.as_str(),
        "S-1-5-18"
            | "S-1-5-32-544"
            | "S-1-5-80-956008885-3418522649-1831038044-1853292631-2271478464"
    ))
}
struct Descriptor(PSECURITY_DESCRIPTOR);
impl Drop for Descriptor {
    fn drop(&mut self) {
        unsafe {
            LocalFree(self.0);
        }
    }
}
pub fn validate(root: &Path, programs: bool) -> Result<()> {
    for (index, directory) in root.ancestors().enumerate() {
        if std::fs::symlink_metadata(directory)?.file_attributes() & 0x400 != 0 {
            return Err(error(
                "Une installation gérée ne peut pas traverser un point de réanalyse.",
            ));
        }
        let name: Vec<u16> = directory.as_os_str().encode_wide().chain(Some(0)).collect();
        let mut owner = std::ptr::null_mut();
        let mut acl = std::ptr::null_mut();
        let mut descriptor = std::ptr::null_mut();
        let code = unsafe {
            GetNamedSecurityInfoW(
                name.as_ptr(),
                SE_FILE_OBJECT,
                OWNER_SECURITY_INFORMATION | DACL_SECURITY_INFORMATION,
                &mut owner,
                std::ptr::null_mut(),
                &mut acl,
                std::ptr::null_mut(),
                &mut descriptor,
            )
        };
        if code != 0 {
            return Err(std::io::Error::from_raw_os_error(code as i32).into());
        }
        let _descriptor = Descriptor(descriptor);
        if owner.is_null() || !trusted(owner)? || acl.is_null() {
            return Err(error(
                "Choisissez un dossier dont les parents sont protégés et appartiennent aux administrateurs ou au système.",
            ));
        }
        let mut info: ACL_SIZE_INFORMATION = unsafe { std::mem::zeroed() };
        if unsafe {
            GetAclInformation(
                acl,
                (&mut info as *mut ACL_SIZE_INFORMATION).cast(),
                std::mem::size_of_val(&info) as u32,
                AclSizeInformation,
            )
        } == 0
        {
            return Err(std::io::Error::last_os_error().into());
        }
        // Ancestors must not permit deleting children or rewriting their DACL.
        // The program directory additionally forbids all file creation/writes.
        let dangerous = 0x10000000
            | WRITE_DAC
            | WRITE_OWNER
            | 0x40
            | if programs && index == 0 {
                0x40000000 | DELETE | 0x2 | 0x4 | 0x10 | 0x100
            } else {
                0
            };
        for i in 0..info.AceCount {
            let mut ace = std::ptr::null_mut();
            if unsafe { GetAce(acl, i, &mut ace) } == 0 {
                return Err(std::io::Error::last_os_error().into());
            }
            let header = unsafe { &*ace.cast::<ACE_HEADER>() };
            if header.AceFlags & INHERIT_ONLY_ACE as u8 != 0 {
                continue;
            }
            if header.AceType == 0 {
                let allowed = unsafe { &*ace.cast::<ACCESS_ALLOWED_ACE>() };
                let sid = (&allowed.SidStart as *const u32).cast_mut().cast();
                if allowed.Mask & dangerous != 0 && !trusted(sid)? {
                    return Err(error(
                        "Le dossier choisi peut être remplacé ou modifié sans élévation. Choisissez un emplacement protégé.",
                    ));
                }
            } else if header.AceType != 1 && header.AceType != 2 && header.AceType != 3 {
                // Do not guess the semantics of object or conditional allow ACEs.
                return Err(error(
                    "Permissions avancées non prises en charge pour l’installation du service.",
                ));
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn windows_program_files_is_a_protected_parent_but_user_temp_is_not() {
        let programs = std::env::var_os("ProgramFiles").unwrap();
        validate(Path::new(&programs), false).unwrap();
        let directory = crate::companion::tests::Directory::new();
        assert!(validate(&directory.0, false).is_err());
    }
}
