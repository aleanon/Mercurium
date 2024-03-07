
#[cfg(windows)]
use windows::{ Win32::Security::Credentials::{self, CRED_TYPE_GENERIC, CRED_PERSIST_LOCAL_MACHINE, CredWriteW},core::PWSTR};


///Stores username and passwords exactly as passed in, remember to hash the password before passing it to the function
/// credentials are stored with identifier "ravault-credentials"
#[cfg(windows)]
pub fn store_credentials(username: String, mut password: String) {
    let mut username = username.encode_utf16().collect::<Vec<u16>>();
    // let password = password.encode_utf16().collect::<Vec<u16>>();
    let mut target_name = "ravault-credentials".encode_utf16().collect::<Vec<u16>>();
    target_name.push(0);


    unsafe {
        let cred = Credentials::CREDENTIALW {
            Type: CRED_TYPE_GENERIC,
            TargetName: PWSTR::from_raw(target_name.as_mut_ptr()),
            CredentialBlob: password.as_mut_ptr(),
            CredentialBlobSize: (password.len() * 2) as u32,
            Persist: CRED_PERSIST_LOCAL_MACHINE,
            UserName: PWSTR::from_raw(username.as_mut_ptr()),
            ..Default::default()
        };

        if let Err(err) = CredWriteW(&cred, 0) {
            // Handle error
            panic!("Unable to store credentials: {err}");
        }
    }
}