#[cfg(target_family = "windows")]
pub mod reg {
    use winreg::RegKey;
    use winreg::enums::HKEY_CURRENT_USER;
    use std::{io, path::Path};

    pub fn write_initialized_flag() -> io::Result<()> {
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let path = Path::new("Software").join("WinEmerge");
        let (key, _) = hkcu.create_subkey(&path)?;
        key.set_value("Initialized", &1u32)?;
        Ok(())
    }

    pub fn read_initialized_flag() -> io::Result<bool> {
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let path = Path::new("Software").join("WinEmerge");
        let key: RegKey = match hkcu.open_subkey(&path) {
            Ok(key) => key,
            Err(_) => return false
        };
        let val: u32 = match key.get_value("Initialized") {
            Ok(val) => val,
            Err(_) => return false
        };

        return val == 1;
    }

}

#[cfg(not(target_family = "windows"))]
pub mod reg {
    use std::{io, env};

    pub fn write_initialized_flag() -> io::Result<()> {
        panic!("Unsupported Platform '{}'", env::consts::OS);
    }

    pub fn read_initialized_flag() -> bool {
        panic!("Unsupported Platform '{}'", env::consts::OS);
    }
}
