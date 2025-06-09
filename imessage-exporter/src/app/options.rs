/*!
 Represents CLI options and validation logic.
*/

use std::path::PathBuf;

use clap::{Arg, ArgAction, ArgMatches, Command, crate_version};

use imessage_database::{
    tables::{attachment::DEFAULT_ATTACHMENT_ROOT, table::DEFAULT_PATH_IOS},
    util::{
        dirs::{default_db_path, home},
        platform::Platform,
        query_context::QueryContext,
    },
};

use crate::app::{
    compatibility::attachment_manager::{AttachmentManager, AttachmentManagerMode},
    error::RuntimeError,
    export_type::ExportType,
};

/// Default export directory name
pub const DEFAULT_OUTPUT_DIR: &str = "imessage_export";

// CLI Arg Names
pub const OPTION_DB_PATH: &str = "db-path";
pub const OPTION_ATTACHMENT_ROOT: &str = "attachment-root";
pub const OPTION_ATTACHMENT_MANAGER: &str = "copy-method";
pub const OPTION_DIAGNOSTIC: &str = "diagnostics";
pub const OPTION_EXPORT_TYPE: &str = "format";
pub const OPTION_EXPORT_PATH: &str = "export-path";
pub const OPTION_START_DATE: &str = "start-date";
pub const OPTION_END_DATE: &str = "end-date";
pub const OPTION_DISABLE_LAZY_LOADING: &str = "no-lazy";
pub const OPTION_CUSTOM_NAME: &str = "custom-name";
pub const OPTION_PLATFORM: &str = "platform";
pub const OPTION_BYPASS_FREE_SPACE_CHECK: &str = "ignore-disk-warning";
pub const OPTION_USE_CALLER_ID: &str = "use-caller-id";
pub const OPTION_CONVERSATION_FILTER: &str = "conversation-filter";
pub const OPTION_CLEARTEXT_PASSWORD: &str = "cleartext-password";

// Other CLI Text
pub const SUPPORTED_FILE_TYPES: &str = "txt, html";
pub const SUPPORTED_PLATFORMS: &str = "macOS, iOS";
pub const SUPPORTED_ATTACHMENT_MANAGER_MODES: &str = "clone, basic, full, disabled";
pub const ABOUT: &str = concat!(
    "The `imessage-exporter` binary exports iMessage data to\n",
    "`txt` or `html` formats. It can also run diagnostics\n",
    "to find problems with the iMessage database."
);

#[derive(Debug, PartialEq, Eq)]
pub struct Options {
    /// Path to database file
    pub db_path: PathBuf,
    /// Custom path to attachments
    pub attachment_root: Option<String>,
    /// The attachment manager type used to copy files
    pub attachment_manager: AttachmentManager,
    /// If true, emit diagnostic information to stdout
    pub diagnostic: bool,
    /// The type of file we are exporting data to
    pub export_type: Option<ExportType>,
    /// Where the app will save exported data
    pub export_path: PathBuf,
    /// Query context describing SQL query filters
    pub query_context: QueryContext,
    /// If true, do not include `loading="lazy"` in HTML exports
    pub no_lazy: bool,
    /// Custom name for database owner in output
    pub custom_name: Option<String>,
    /// If true, use the database owner's caller ID instead of "Me"
    pub use_caller_id: bool,
    /// The database source's platform
    pub platform: Platform,
    /// If true, disable the free disk space check
    pub ignore_disk_space: bool,
    /// An optional filter for conversation participants
    pub conversation_filter: Option<String>,
    /// An optional password for encrypted backups
    pub cleartext_password: Option<String>,
}

impl Options {
    pub fn from_args(args: &ArgMatches) -> Result<Self, RuntimeError> {
        let user_path: Option<&String> = args.get_one(OPTION_DB_PATH);
        let attachment_root: Option<&String> = args.get_one(OPTION_ATTACHMENT_ROOT);
        let attachment_manager_type: Option<&String> = args.get_one(OPTION_ATTACHMENT_MANAGER);
        let diagnostic = args.get_flag(OPTION_DIAGNOSTIC);
        let export_file_type: Option<&String> = args.get_one(OPTION_EXPORT_TYPE);
        let user_export_path: Option<&String> = args.get_one(OPTION_EXPORT_PATH);
        let start_date: Option<&String> = args.get_one(OPTION_START_DATE);
        let end_date: Option<&String> = args.get_one(OPTION_END_DATE);
        let no_lazy = args.get_flag(OPTION_DISABLE_LAZY_LOADING);
        let custom_name: Option<&String> = args.get_one(OPTION_CUSTOM_NAME);
        let use_caller_id = args.get_flag(OPTION_USE_CALLER_ID);
        let platform_type: Option<&String> = args.get_one(OPTION_PLATFORM);
        let ignore_disk_space = args.get_flag(OPTION_BYPASS_FREE_SPACE_CHECK);
        let conversation_filter: Option<&String> = args.get_one(OPTION_CONVERSATION_FILTER);
        let cleartext_password: Option<&String> = args.get_one(OPTION_CLEARTEXT_PASSWORD);

        // Build the export type
        let export_type: Option<ExportType> = match export_file_type {
            Some(export_type_str) => {
                Some(ExportType::from_cli(export_type_str).ok_or(RuntimeError::InvalidOptions(format!(
                    "{export_type_str} is not a valid export type! Must be one of <{SUPPORTED_FILE_TYPES}>"
                )))?)
            }
            None => None,
        };

        // Anything in here requires `--format`
        if export_file_type.is_none() {
            let format_deps = [
                (attachment_manager_type.is_some(), OPTION_ATTACHMENT_MANAGER),
                (user_export_path.is_some(), OPTION_EXPORT_PATH),
                (no_lazy, OPTION_DISABLE_LAZY_LOADING),
                (start_date.is_some(), OPTION_START_DATE),
                (end_date.is_some(), OPTION_END_DATE),
                (custom_name.is_some(), OPTION_CUSTOM_NAME),
                (use_caller_id, OPTION_USE_CALLER_ID),
                (conversation_filter.is_some(), OPTION_CONVERSATION_FILTER),
            ];
            for (set, opt) in format_deps {
                if set {
                    return Err(RuntimeError::InvalidOptions(format!(
                        "Option --{opt} is enabled, which requires --{OPTION_EXPORT_TYPE}"
                    )));
                }
            }
        }

        // During `diagnostics`, none of these may be set
        let diag_conflicts = [
            (attachment_manager_type.is_some(), OPTION_ATTACHMENT_MANAGER),
            (user_export_path.is_some(), OPTION_EXPORT_PATH),
            (no_lazy, OPTION_DISABLE_LAZY_LOADING),
            (export_file_type.is_some(), OPTION_EXPORT_TYPE),
            (start_date.is_some(), OPTION_START_DATE),
            (end_date.is_some(), OPTION_END_DATE),
            (use_caller_id, OPTION_USE_CALLER_ID),
            (custom_name.is_some(), OPTION_CUSTOM_NAME),
            (conversation_filter.is_some(), OPTION_CONVERSATION_FILTER),
        ];
        for (set, opt) in diag_conflicts {
            if diagnostic && set {
                return Err(RuntimeError::InvalidOptions(format!(
                    "Diagnostics are enabled; `{opt}` is disallowed"
                )));
            }
        }

        // Prevent custom_name vs. use_caller_id collision
        if custom_name.is_some() && use_caller_id {
            return Err(RuntimeError::InvalidOptions(format!(
                "--{OPTION_CUSTOM_NAME} is enabled; --{OPTION_USE_CALLER_ID} is disallowed"
            )));
        }

        // Build query context
        let mut query_context = QueryContext::default();
        if let Some(start) = start_date {
            if let Err(why) = query_context.set_start(start) {
                return Err(RuntimeError::InvalidOptions(format!("{why}")));
            }
        }
        if let Some(end) = end_date {
            if let Err(why) = query_context.set_end(end) {
                return Err(RuntimeError::InvalidOptions(format!("{why}")));
            }
        }

        // We have to allocate a PathBuf here because it can be created from data owned by this function in the default state
        let db_path = match user_path {
            Some(path) => PathBuf::from(path),
            None => default_db_path(),
        };

        // Build the Platform
        let platform = match platform_type {
            Some(platform_str) => {
                Platform::from_cli(platform_str).ok_or(RuntimeError::InvalidOptions(format!(
                    "{platform_str} is not a valid platform! Must be one of <{SUPPORTED_PLATFORMS}>"
                )))?
            }
            None => Platform::determine(&db_path)?,
        };

        // Prevent cleartext_password from being set if the source is not an iOS backup
        if cleartext_password.is_some() && !matches!(platform, Platform::iOS) {
            return Err(RuntimeError::InvalidOptions(format!(
                "--{OPTION_CLEARTEXT_PASSWORD} is enabled; it can only be used with iOS backups."
            )));
        }

        // Validate that the custom attachment root exists, if provided
        if let Some(path) = attachment_root {
            let custom_attachment_path = PathBuf::from(path);
            if !custom_attachment_path.exists() {
                return Err(RuntimeError::InvalidOptions(format!(
                    "Supplied {OPTION_ATTACHMENT_ROOT} `{path}` does not exist!"
                )));
            }
        }

        // Warn the user that custom attachment roots have no effect on iOS backups
        if attachment_root.is_some() && platform == Platform::iOS {
            eprintln!(
                "Option {OPTION_ATTACHMENT_ROOT} is enabled, but the platform is {}, so the root will have no effect!",
                Platform::iOS
            );
        }

        // Determine the attachment manager mode
        let attachment_manager_mode = match attachment_manager_type {
            Some(manager) => {
                AttachmentManagerMode::from_cli(manager).ok_or(RuntimeError::InvalidOptions(format!(
                    "{manager} is not a valid attachment manager mode! Must be one of <{SUPPORTED_ATTACHMENT_MANAGER_MODES}>"
                )))?
            }
            None => AttachmentManagerMode::default(),
        };

        // Validate the provided export path
        let export_path = validate_path(user_export_path, &export_type.as_ref())?;

        Ok(Options {
            db_path,
            attachment_root: attachment_root.cloned(),
            attachment_manager: AttachmentManager::from(attachment_manager_mode),
            diagnostic,
            export_type,
            export_path,
            query_context,
            no_lazy,
            custom_name: custom_name.cloned(),
            use_caller_id,
            platform,
            ignore_disk_space,
            conversation_filter: conversation_filter.cloned(),
            cleartext_password: cleartext_password.cloned(),
        })
    }

    /// Generate a path to the database based on the currently selected platform
    pub fn get_db_path(&self) -> PathBuf {
        match self.platform {
            Platform::iOS => self.db_path.join(DEFAULT_PATH_IOS),
            Platform::macOS => self.db_path.clone(),
        }
    }
}

/// Ensure export path is empty or does not contain files of the existing export type
///
/// We have to allocate a `PathBuf` here because it can be created from data owned by this function in the default state
fn validate_path(
    export_path: Option<&String>,
    export_type: &Option<&ExportType>,
) -> Result<PathBuf, RuntimeError> {
    // Build a path from the user-provided data or the default location
    let resolved_path =
        PathBuf::from(export_path.unwrap_or(&format!("{}/{DEFAULT_OUTPUT_DIR}", home())));

    // If there is an export type selected, ensure we do not overwrite files of the same type
    if let Some(export_type) = export_type {
        if resolved_path.exists() {
            // Get the word to use if there is a problem with the specified path
            let path_word = match export_path {
                Some(_) => "Specified",
                None => "Default",
            };

            // Ensure the directory exists and does not contain files of the same export type
            match resolved_path.read_dir() {
                Ok(files) => {
                    let export_type_extension = export_type.to_string();
                    for file in files.flatten() {
                        if file
                            .path()
                            .extension()
                            .is_some_and(|s| s.to_str().unwrap_or("") == export_type_extension)
                        {
                            return Err(RuntimeError::InvalidOptions(format!(
                                "{path_word} export path {resolved_path:?} contains existing \"{export_type}\" export data!"
                            )));
                        }
                    }
                }
                Err(why) => {
                    return Err(RuntimeError::InvalidOptions(format!(
                        "{path_word} export path {resolved_path:?} is not a valid directory: {why}"
                    )));
                }
            }
        }
    }

    Ok(resolved_path)
}

/// Build the command line argument parser
fn get_command() -> Command {
    Command::new("iMessage Exporter")
        .version(crate_version!())
        .about(ABOUT)
        .arg_required_else_help(true)
        .arg(
            Arg::new(OPTION_DIAGNOSTIC)
            .short('d')
            .long(OPTION_DIAGNOSTIC)
            .help("Print diagnostic information and exit\n")
            .action(ArgAction::SetTrue)
            .display_order(0),
        )
        .arg(
            Arg::new(OPTION_EXPORT_TYPE)
            .short('f')
            .long(OPTION_EXPORT_TYPE)
            .help("Specify a single file format to export messages into\n")
            .display_order(1)
            .value_name(SUPPORTED_FILE_TYPES),
        )
        .arg(
            Arg::new(OPTION_ATTACHMENT_MANAGER)
            .short('c')
            .long(OPTION_ATTACHMENT_MANAGER)
            .help(format!("Specify an optional method to use when copying message attachments\n`clone` will copy all files without converting anything\n`basic` will copy all files and convert HEIC images to JPEG\n`full` will copy all files and convert HEIC files to JPEG, CAF to MP4, and MOV to MP4\nIf omitted, the default is `{}`\nImageMagick is required to convert images on non-macOS platforms\nffmpeg is required to convert audio on non-macOS platforms and video on all platforms\n", AttachmentManagerMode::default()))
            .display_order(2)
            .value_name(SUPPORTED_ATTACHMENT_MANAGER_MODES),
        )
        .arg(
            Arg::new(OPTION_DB_PATH)
                .short('p')
                .long(OPTION_DB_PATH)
                .help(format!("Specify an optional custom path for the iMessage database location\nFor macOS, specify a path to a `chat.db` file\nFor iOS, specify a path to the root of a device backup directory\nIf the iOS backup is encrypted, --{OPTION_CLEARTEXT_PASSWORD} must be passed\nIf omitted, the default directory is {}\n", default_db_path().display()))
                .display_order(3)
                .value_name("path/to/source"),
        )
        .arg(
            Arg::new(OPTION_ATTACHMENT_ROOT)
                .short('r')
                .long(OPTION_ATTACHMENT_ROOT)
                .help(format!("Specify an optional custom path to look for attachments in (macOS only)\nOnly use this if attachments are stored separately from the database's default location\nThe default location is {}\n", DEFAULT_ATTACHMENT_ROOT.replacen('~', &home(), 1)))
                .display_order(4)
                .value_name("path/to/attachments"),
        )
        .arg(
            Arg::new(OPTION_PLATFORM)
            .short('a')
            .long(OPTION_PLATFORM)
            .help("Specify the platform the database was created on\nIf omitted, the platform type is determined automatically\n")
            .display_order(5)
            .value_name(SUPPORTED_PLATFORMS),
        )
        .arg(
            Arg::new(OPTION_EXPORT_PATH)
                .short('o')
                .long(OPTION_EXPORT_PATH)
                .help(format!("Specify an optional custom directory for outputting exported data\nIf omitted, the default directory is {}/{DEFAULT_OUTPUT_DIR}\n", home()))
                .display_order(6)
                .value_name("path/to/save/files"),
        )
        .arg(
            Arg::new(OPTION_START_DATE)
                .short('s')
                .long(OPTION_START_DATE)
                .help("The start date filter\nOnly messages sent on or after this date will be included\n")
                .display_order(7)
                .value_name("YYYY-MM-DD"),
        )
        .arg(
            Arg::new(OPTION_END_DATE)
                .short('e')
                .long(OPTION_END_DATE)
                .help("The end date filter\nOnly messages sent before this date will be included\n")
                .display_order(8)
                .value_name("YYYY-MM-DD"),
        )
        .arg(
            Arg::new(OPTION_DISABLE_LAZY_LOADING)
                .short('l')
                .long(OPTION_DISABLE_LAZY_LOADING)
                .help("Do not include `loading=\"lazy\"` in HTML export `img` tags\nThis will make pages load slower but PDF generation work\n")
                .action(ArgAction::SetTrue)
                .display_order(9),
        )
        .arg(
            Arg::new(OPTION_CUSTOM_NAME)
                .short('m')
                .long(OPTION_CUSTOM_NAME)
                .help(format!("Specify an optional custom name for the database owner's messages in exports\nConflicts with --{OPTION_USE_CALLER_ID}\n"))
                .display_order(10)
        )
        .arg(
            Arg::new(OPTION_USE_CALLER_ID)
                .short('i')
                .long(OPTION_USE_CALLER_ID)
                .help(format!("Use the database owner's caller ID in exports instead of \"Me\"\nConflicts with --{OPTION_CUSTOM_NAME}\n"))
                .action(ArgAction::SetTrue)
                .display_order(11)
        )
        .arg(
            Arg::new(OPTION_BYPASS_FREE_SPACE_CHECK)
                .short('b')
                .long(OPTION_BYPASS_FREE_SPACE_CHECK)
                .help("Bypass the disk space check when exporting data\nBy default, exports will not run if there is not enough free disk space\n")
                .action(ArgAction::SetTrue)
                .display_order(12)
        )
        .arg(
            Arg::new(OPTION_CONVERSATION_FILTER)
                .short('t')
                .long(OPTION_CONVERSATION_FILTER)
                .help("Filter exported conversations by contact numbers or emails\nTo provide multiple filter criteria, use a comma-separated string\nAll conversations with the specified participants are exported, including group conversations\nExample: `-t steve@apple.com,5558675309`\n")
                .display_order(13)
                .value_name("filter"),
        )
        .arg(
            Arg::new(OPTION_CLEARTEXT_PASSWORD)
                .short('x')
                .long(OPTION_CLEARTEXT_PASSWORD)
                .help("Optional password for encrypted iOS backups\nThis is only used when the source is an encrypted iOS backup directory\n")
                .display_order(14)
                .value_name("password"),
        )
}

#[cfg(test)]
impl Options {
    pub fn fake_options(export_type: ExportType) -> Options {
        Options {
            db_path: std::env::current_dir()
                .unwrap()
                .parent()
                .unwrap()
                .join("imessage-database/test_data/db/test.db"),
            attachment_root: None,
            attachment_manager: AttachmentManager::default(),
            diagnostic: false,
            export_type: Some(export_type),
            export_path: PathBuf::from("/tmp"),
            query_context: QueryContext::default(),
            no_lazy: false,
            custom_name: None,
            use_caller_id: false,
            platform: Platform::macOS,
            ignore_disk_space: false,
            conversation_filter: None,
            cleartext_password: None,
        }
    }
}

/// Parse arguments from the command line
pub fn from_command_line() -> ArgMatches {
    get_command().get_matches()
}

#[cfg(test)]
mod arg_tests {
    use std::fs;

    use imessage_database::util::{
        dirs::default_db_path, platform::Platform, query_context::QueryContext,
    };

    use crate::app::{
        compatibility::attachment_manager::{AttachmentManager, AttachmentManagerMode},
        export_type::ExportType,
        options::{Options, get_command, validate_path},
    };

    #[test]
    fn can_build_option_diagnostic_flag() {
        // Get matches from sample args
        let command = get_command();
        let args = command.get_matches_from(["imessage-exporter", "-d"]);

        // Build the Options
        let actual = Options::from_args(&args).unwrap();

        // Expected data
        let expected = Options {
            db_path: default_db_path(),
            attachment_root: None,
            attachment_manager: AttachmentManager::from(AttachmentManagerMode::Disabled),
            diagnostic: true,
            export_type: None,
            export_path: validate_path(None, &None).unwrap(),
            query_context: QueryContext::default(),
            no_lazy: false,
            custom_name: None,
            use_caller_id: false,
            platform: Platform::default(),
            ignore_disk_space: false,
            conversation_filter: None,
            cleartext_password: None,
        };

        assert_eq!(actual, expected);
    }

    #[test]
    fn cant_build_option_diagnostic_flag_with_export_type() {
        // Get matches from sample args
        let command = get_command();
        let args = command.get_matches_from(["imessage-exporter", "-d", "-f", "txt"]);
        assert!(Options::from_args(&args).is_err());
    }

    #[test]
    fn cant_build_option_diagnostic_flag_with_export_path() {
        // Get matches from sample args
        let command = get_command();
        let args = command.get_matches_from(["imessage-exporter", "-d", "-o", "~/test"]);
        assert!(Options::from_args(&args).is_err());
    }

    #[test]
    fn cant_build_option_diagnostic_flag_with_attachment_manager() {
        // Get matches from sample args
        let command = get_command();
        let args = command.get_matches_from(["imessage-exporter", "-d", "-c", "basic"]);
        assert!(Options::from_args(&args).is_err());
    }

    #[test]
    fn cant_build_option_diagnostic_flag_with_start_date() {
        // Get matches from sample args
        let command = get_command();
        let args = command.get_matches_from(["imessage-exporter", "-d", "-s", "2020-01-01"]);
        assert!(Options::from_args(&args).is_err());
    }

    #[test]
    fn cant_build_option_diagnostic_flag_with_end() {
        // Get matches from sample args
        let command = get_command();
        let args = command.get_matches_from(["imessage-exporter", "-d", "-e", "2020-01-01"]);
        assert!(Options::from_args(&args).is_err());
    }

    #[test]
    fn cant_build_option_diagnostic_flag_with_caller_id() {
        // Get matches from sample args
        let command = get_command();
        let args = command.get_matches_from(["imessage-exporter", "-d", "-i"]);
        assert!(Options::from_args(&args).is_err());
    }

    #[test]
    fn can_build_option_export_html() {
        // Cleanup existing temp data
        let _ = fs::remove_file("/tmp/orphaned.html");

        // Get matches from sample args
        let command = get_command();
        let args = command.get_matches_from(["imessage-exporter", "-f", "html", "-o", "/tmp"]);

        // Build the Options
        let actual = Options::from_args(&args).unwrap();

        // Expected data
        let tmp_dir = String::from("/tmp");
        let expected = Options {
            db_path: default_db_path(),
            attachment_root: None,
            attachment_manager: AttachmentManager::from(AttachmentManagerMode::Disabled),
            diagnostic: false,
            export_type: Some(ExportType::Html),
            export_path: validate_path(Some(&tmp_dir), &None).unwrap(),
            query_context: QueryContext::default(),
            no_lazy: false,
            custom_name: None,
            use_caller_id: false,
            platform: Platform::default(),
            ignore_disk_space: false,
            conversation_filter: None,
            cleartext_password: None,
        };

        assert_eq!(actual, expected);
    }

    #[test]
    fn can_build_option_export_txt_no_lazy() {
        // Cleanup existing temp data
        let _ = fs::remove_file("/tmp/orphaned.txt");

        // Get matches from sample args
        let command = get_command();
        let args = command.get_matches_from(["imessage-exporter", "-f", "txt", "-l"]);

        // Build the Options
        let actual = Options::from_args(&args).unwrap();

        // Expected data
        let expected = Options {
            db_path: default_db_path(),
            attachment_root: None,
            attachment_manager: AttachmentManager::from(AttachmentManagerMode::Disabled),
            diagnostic: false,
            export_type: Some(ExportType::Txt),
            export_path: validate_path(None, &None).unwrap(),
            query_context: QueryContext::default(),
            no_lazy: true,
            custom_name: None,
            use_caller_id: false,
            platform: Platform::default(),
            ignore_disk_space: false,
            conversation_filter: None,
            cleartext_password: None,
        };

        assert_eq!(actual, expected);
    }

    #[test]
    fn cant_build_option_attachment_manager_no_export_type() {
        // Get matches from sample args
        let command = get_command();
        let args = command.get_matches_from(["imessage-exporter", "-c", "clone"]);
        assert!(Options::from_args(&args).is_err());
    }

    #[test]
    fn cant_build_option_export_path_no_export_type() {
        // Get matches from sample args
        let command = get_command();
        let args = command.get_matches_from(["imessage-exporter", "-o", "~/test"]);
        assert!(Options::from_args(&args).is_err());
    }

    #[test]
    fn cant_build_option_start_date_path_no_export_type() {
        // Get matches from sample args
        let command = get_command();
        let args = command.get_matches_from(["imessage-exporter", "-s", "2020-01-01"]);
        assert!(Options::from_args(&args).is_err());
    }

    #[test]
    fn cant_build_option_end_date_path_no_export_type() {
        // Get matches from sample args
        let command = get_command();
        let args = command.get_matches_from(["imessage-exporter", "-e", "2020-01-01"]);
        assert!(Options::from_args(&args).is_err());
    }

    #[test]
    fn cant_build_option_invalid_date() {
        // Get matches from sample args
        let command = get_command();
        let args =
            command.get_matches_from(["imessage-exporter", "-f", "html", "-e", "2020-32-32"]);
        assert!(Options::from_args(&args).is_err());
    }

    #[test]
    fn cant_build_option_invalid_platform() {
        // Get matches from sample args
        let command = get_command();
        let args = command.get_matches_from(["imessage-exporter", "-a", "iPad"]);
        assert!(Options::from_args(&args).is_err());
    }

    #[test]
    fn can_build_option_valid_platform() {
        // Get matches from sample args
        let command = get_command();
        let args = command.get_matches_from(["imessage-exporter", "-a", "ios", "-f", "txt"]);

        // Build the Options
        let actual = Options::from_args(&args).unwrap();

        // Expected data
        let expected = Options {
            db_path: default_db_path(),
            attachment_root: None,
            attachment_manager: AttachmentManager::from(AttachmentManagerMode::Disabled),
            diagnostic: false,
            export_type: Some(ExportType::Txt),
            export_path: validate_path(None, &None).unwrap(),
            query_context: QueryContext::default(),
            no_lazy: false,
            custom_name: None,
            use_caller_id: false,
            platform: Platform::iOS,
            ignore_disk_space: false,
            conversation_filter: None,
            cleartext_password: None,
        };

        assert_eq!(actual, expected);
    }

    #[test]
    fn can_build_option_ios_password() {
        // Get matches from sample args
        let command = get_command();
        let args = command.get_matches_from([
            "imessage-exporter",
            "-a",
            "ios",
            "-f",
            "txt",
            "-x",
            "password",
        ]);

        // Build the Options
        let actual = Options::from_args(&args).unwrap();

        // Expected data
        let expected = Options {
            db_path: default_db_path(),
            attachment_root: None,
            attachment_manager: AttachmentManager::from(AttachmentManagerMode::Disabled),
            diagnostic: false,
            export_type: Some(ExportType::Txt),
            export_path: validate_path(None, &None).unwrap(),
            query_context: QueryContext::default(),
            no_lazy: false,
            custom_name: None,
            use_caller_id: false,
            platform: Platform::iOS,
            ignore_disk_space: false,
            conversation_filter: None,
            cleartext_password: Some("password".to_string()),
        };

        assert_eq!(actual, expected);
    }

    #[test]
    fn cant_build_option_macos_password() {
        // Get matches from sample args
        let command = get_command();
        let args = command.get_matches_from([
            "imessage-exporter",
            "-a",
            "macos",
            "-f",
            "txt",
            "-x",
            "password",
        ]);
        assert!(Options::from_args(&args).is_err());
    }

    #[test]
    fn cant_build_option_invalid_export_type() {
        // Get matches from sample args
        let command = get_command();
        let args = command.get_matches_from(["imessage-exporter", "-f", "pdf"]);
        assert!(Options::from_args(&args).is_err());
    }

    #[test]
    fn can_build_option_custom_name() {
        // Get matches from sample args
        let command = get_command();
        let args = command.get_matches_from(["imessage-exporter", "-f", "txt", "-m", "Name"]);

        // Build the Options
        let actual = Options::from_args(&args).unwrap();

        // Expected data
        let expected = Options {
            db_path: default_db_path(),
            attachment_root: None,
            attachment_manager: AttachmentManager::from(AttachmentManagerMode::Disabled),
            diagnostic: false,
            export_type: Some(ExportType::Txt),
            export_path: validate_path(None, &None).unwrap(),
            query_context: QueryContext::default(),
            no_lazy: false,
            custom_name: Some("Name".to_string()),
            use_caller_id: false,
            platform: Platform::default(),
            ignore_disk_space: false,
            conversation_filter: None,
            cleartext_password: None,
        };

        assert_eq!(actual, expected);
    }

    #[test]
    fn can_build_option_caller_id() {
        // Get matches from sample args
        let command = get_command();
        let args = command.get_matches_from(["imessage-exporter", "-f", "txt", "-i"]);

        // Build the Options
        let actual = Options::from_args(&args).unwrap();

        // Expected data
        let expected = Options {
            db_path: default_db_path(),
            attachment_root: None,
            attachment_manager: AttachmentManager::from(AttachmentManagerMode::Disabled),
            diagnostic: false,
            export_type: Some(ExportType::Txt),
            export_path: validate_path(None, &None).unwrap(),
            query_context: QueryContext::default(),
            no_lazy: false,
            custom_name: None,
            use_caller_id: true,
            platform: Platform::default(),
            ignore_disk_space: false,
            conversation_filter: None,
            cleartext_password: None,
        };

        assert_eq!(actual, expected);
    }

    #[test]
    fn can_build_option_contact_filter() {
        // Get matches from sample args
        let command = get_command();
        let args =
            command.get_matches_from(["imessage-exporter", "-t", "steve@apple.com", "-f", "txt"]);

        // Build the Options
        let actual = Options::from_args(&args).unwrap();

        // Expected data
        let expected = Options {
            db_path: default_db_path(),
            attachment_root: None,
            attachment_manager: AttachmentManager::from(AttachmentManagerMode::Disabled),
            diagnostic: false,
            export_type: Some(ExportType::Txt),
            export_path: validate_path(None, &None).unwrap(),
            query_context: QueryContext::default(),
            no_lazy: false,
            custom_name: None,
            use_caller_id: false,
            platform: Platform::default(),
            ignore_disk_space: false,
            conversation_filter: Some(String::from("steve@apple.com")),
            cleartext_password: None,
        };

        assert_eq!(actual, expected);
    }

    #[test]
    fn can_build_option_full() {
        // Get matches from sample args
        let command = get_command();
        let args = command.get_matches_from(["imessage-exporter", "-f", "txt", "-c", "full"]);

        // Build the Options
        let actual = Options::from_args(&args).unwrap();

        // Expected data
        let expected = Options {
            db_path: default_db_path(),
            attachment_root: None,
            attachment_manager: AttachmentManager::from(AttachmentManagerMode::Full),
            diagnostic: false,
            export_type: Some(ExportType::Txt),
            export_path: validate_path(None, &None).unwrap(),
            query_context: QueryContext::default(),
            no_lazy: false,
            custom_name: None,
            use_caller_id: false,
            platform: Platform::default(),
            ignore_disk_space: false,
            conversation_filter: None,
            cleartext_password: None,
        };

        assert_eq!(actual, expected);
    }

    #[test]
    fn can_build_option_clone() {
        // Get matches from sample args
        let command = get_command();
        let args = command.get_matches_from(["imessage-exporter", "-f", "txt", "-c", "clone"]);

        // Build the Options
        let actual = Options::from_args(&args).unwrap();

        // Expected data
        let expected = Options {
            db_path: default_db_path(),
            attachment_root: None,
            attachment_manager: AttachmentManager::from(AttachmentManagerMode::Clone),
            diagnostic: false,
            export_type: Some(ExportType::Txt),
            export_path: validate_path(None, &None).unwrap(),
            query_context: QueryContext::default(),
            no_lazy: false,
            custom_name: None,
            use_caller_id: false,
            platform: Platform::default(),
            ignore_disk_space: false,
            conversation_filter: None,
            cleartext_password: None,
        };

        assert_eq!(actual, expected);
    }

    #[test]
    fn cant_build_option_custom_name_and_caller_id() {
        // Get matches from sample args
        let command = get_command();
        let args = command.get_matches_from(["imessage-exporter", "-f", "txt", "-m", "Name", "-i"]);
        assert!(Options::from_args(&args).is_err());
    }

    #[test]
    fn cant_build_option_caller_id_no_export() {
        // Get matches from sample args
        let command = get_command();
        let args = command.get_matches_from(["imessage-exporter", "-i"]);
        assert!(Options::from_args(&args).is_err());
    }

    #[test]
    fn cant_build_option_custom_name_no_export() {
        // Get matches from sample args
        let command = get_command();
        let args = command.get_matches_from(["imessage-exporter", "-m", "Name"]);
        assert!(Options::from_args(&args).is_err());
    }

    #[test]
    fn cant_build_option_contact_filter_no_export() {
        // Get matches from sample args
        let command = get_command();
        let args = command.get_matches_from(["imessage-exporter", "-t", "steve@apple.com"]);
        assert!(Options::from_args(&args).is_err());
    }

    #[test]
    fn cant_build_option_no_lazy_without_format() {
        let args = get_command().get_matches_from(["imessage-exporter", "-l"]);
        assert!(Options::from_args(&args).is_err());
    }

    #[test]
    fn cant_build_option_no_lazy_with_diagnostics() {
        let args = get_command().get_matches_from(["imessage-exporter", "-d", "-l"]);
        assert!(Options::from_args(&args).is_err());
    }

    #[test]
    fn can_build_option_ignore_disk_space_flag() {
        let args = get_command().get_matches_from(["imessage-exporter", "-f", "txt", "-b"]);

        // Build the Options
        let actual = Options::from_args(&args).unwrap();

        // Expected data
        let expected = Options {
            db_path: default_db_path(),
            attachment_root: None,
            attachment_manager: AttachmentManager::from(AttachmentManagerMode::Disabled),
            diagnostic: false,
            export_type: Some(ExportType::Txt),
            export_path: validate_path(None, &None).unwrap(),
            query_context: QueryContext::default(),
            no_lazy: false,
            custom_name: None,
            use_caller_id: false,
            platform: Platform::default(),
            ignore_disk_space: true,
            conversation_filter: None,
            cleartext_password: None,
        };

        assert_eq!(actual, expected);
    }

    #[test]
    fn cant_build_option_invalid_attachment_root() {
        let args = get_command().get_matches_from(["imessage-exporter", "-r", "/does/not/exist"]);
        assert!(Options::from_args(&args).is_err());
    }
}

#[cfg(test)]
mod path_tests {
    use std::fs;
    use std::io::Write;
    use std::path::PathBuf;

    use crate::app::{
        export_type::ExportType,
        options::{DEFAULT_OUTPUT_DIR, validate_path},
    };
    use imessage_database::util::dirs::home;

    #[test]
    fn can_validate_empty() {
        // Cleanup existing temp data
        let _ = fs::remove_file("/tmp/orphaned.txt");

        let tmp = String::from("/tmp");
        let export_path = Some(&tmp);
        let export_type = Some(ExportType::Txt);

        let result = validate_path(export_path, &export_type.as_ref());

        assert_eq!(result.unwrap(), PathBuf::from("/tmp"));
    }

    #[test]
    fn can_validate_different_type() {
        // Cleanup existing temp data
        let _ = fs::remove_file("/tmp/orphaned.txt");

        let tmp = String::from("/tmp");
        let export_path = Some(&tmp);
        let export_type = Some(ExportType::Txt);

        let result = validate_path(export_path, &export_type.as_ref());

        let mut tmp = PathBuf::from("/tmp");
        tmp.push("fake1.html");
        let mut file = fs::File::create(&tmp).unwrap();
        file.write_all(&[]).unwrap();

        assert_eq!(result.unwrap(), PathBuf::from("/tmp"));
        fs::remove_file(&tmp).unwrap();
    }

    #[test]
    fn can_validate_same_type() {
        // Cleanup existing temp data
        let _ = fs::remove_file("/tmp/orphaned.txt");

        let tmp = String::from("/tmp");
        let export_path = Some(&tmp);
        let export_type = Some(ExportType::Txt);

        let result = validate_path(export_path, &export_type.as_ref());

        let mut tmp = PathBuf::from("/tmp");
        tmp.push("fake2.txt");
        let mut file = fs::File::create(&tmp).unwrap();
        file.write_all(&[]).unwrap();

        assert_eq!(result.unwrap(), PathBuf::from("/tmp"));
        fs::remove_file(&tmp).unwrap();
    }

    #[test]
    fn can_validate_none() {
        let export_path = None;
        let export_type = None;

        let result = validate_path(export_path, &export_type);

        assert_eq!(
            result.unwrap(),
            PathBuf::from(&format!("{}/{DEFAULT_OUTPUT_DIR}", home()))
        );
    }
}
