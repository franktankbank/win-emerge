#[cfg(target_family = "windows")]
pub mod reg {
    use winreg::HKCU;
    use std::{io, path::Path};

    pub fn write_initialized_flag() -> io::Result<()> {
        let path = Path::new("Software").join("WinEmerge");
        let (key, _) = HKCU.create_subkey(&path)?;
        key.set_value("Initialized", &1u32)?;
        Ok(())
    }

    pub fn read_initialized_flag() -> bool {
        let path = Path::new("Software").join("WinEmerge");
        let key = match HKCU.open_subkey(&path) {
            Ok(key) => key,
            Err(_) => return false
        };
        let val = match key.get_value("Initialized") {
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
