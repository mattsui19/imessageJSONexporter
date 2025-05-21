use std::{env::temp_dir, fs::File, io::copy, path::PathBuf};

use crabapple::{Authentication, Backup};
use imessage_database::{tables::table::DEFAULT_PATH_IOS, util::platform::Platform};

use crate::app::{error::RuntimeError, options::Options};

/// Decrypt the iOS backup, if necessary
pub fn decrypt_backup(options: &Options) -> Result<Option<Backup>, RuntimeError> {
    let (Platform::iOS, Some(pw)) = (&options.platform, &options.cleartext_password) else {
        return Ok(None);
    };

    eprintln!("Decrypting iOS backup...");
    eprintln!("  [1/3] Deriving backup keys...");
    let auth = Authentication::Password(pw.clone());
    let backup = Backup::new(options.db_path.clone(), &auth)?;

    Ok(Some(backup))
}

pub fn get_decrypted_message_database(backup: &Backup) -> Result<PathBuf, RuntimeError> {
    let (_, file_id) = DEFAULT_PATH_IOS.split_at(3);
    eprintln!("  [2/3] Resolving `sms.db`...");
    let file = backup.get_file(file_id)?;

    let mut decrypted_chat_db = backup.decrypt_entry_stream(&file)?;

    // Write decrypted sms.db into a platform-specific temporary directory
    let tmp_path = temp_dir().join("crabapple-sms.db");
    let mut file = File::create(&tmp_path).map_err(RuntimeError::DiskError)?;

    // Stream-decrypt directly into the temp file
    eprintln!("  [3/3] Decrypting `sms.db`...");
    copy(&mut decrypted_chat_db, &mut file).map_err(RuntimeError::DiskError)?;

    eprintln!(
        "Decrypted iOS backup: {} (version {})\n",
        backup.lockdown().device_name,
        backup.lockdown().product_version,
    );
    Ok(tmp_path)
}
