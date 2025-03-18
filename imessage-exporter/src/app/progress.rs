/*!
 Defines the export progress bar.
*/

use std::time::Duration;

use indicatif::{ProgressBar, ProgressDrawTarget, ProgressStyle};

const TEMPLATE_DEFAULT: &str =
    "{spinner:.green} [{elapsed}] [{bar:.blue}] {human_pos}/{human_len} ({per_sec}, ETA: {eta})";
const TEMPLATE_BUSY: &str =
    "{spinner:.green} [{elapsed}] [{bar:.blue}] {human_pos}/{human_len} ({msg})";

pub fn build_progress_bar_export() -> ProgressBar {
    let pb = ProgressBar::hidden();
    pb.set_style(
        ProgressStyle::default_bar()
            .template(TEMPLATE_DEFAULT)
            .unwrap()
            .progress_chars("#>-"),
    );
    pb
}

pub fn start_progress_bar(pb: &ProgressBar, length: u64) {
    pb.set_position(0);
    pb.enable_steady_tick(Duration::from_millis(100));
    pb.set_length(length);
    pb.set_draw_target(ProgressDrawTarget::stdout());
}

pub fn set_progress_bar_default(pb: &ProgressBar) {
    pb.set_style(
        ProgressStyle::default_bar()
            .template(TEMPLATE_DEFAULT)
            .unwrap()
            .progress_chars("#>-"),
    );
    pb.enable_steady_tick(Duration::from_millis(100));
}

pub fn set_progress_bar_busy(pb: &ProgressBar, message: String) {
    pb.set_style(
        ProgressStyle::default_bar()
            .template(TEMPLATE_BUSY)
            .unwrap()
            .progress_chars("#>-"),
    );
    pb.set_message(message);
    pb.enable_steady_tick(Duration::from_millis(250));
}
