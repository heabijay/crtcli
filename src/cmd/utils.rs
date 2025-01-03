use std::path::PathBuf;
use time::macros::format_description;

pub fn get_next_filename_if_exists(path_buf: PathBuf) -> PathBuf {
    if !path_buf.exists() {
        return path_buf;
    };

    if path_buf.file_name().is_none() {
        return path_buf;
    };

    let (path_buf_stem, path_buf_extension) = {
        let filename = path_buf.file_name().unwrap().to_str().unwrap();
        let ext_index = if path_buf.is_dir() {
            None
        } else {
            filename.rfind('.')
        };

        if let Some(ext_index) = ext_index {
            (&filename[..ext_index], &filename[ext_index..])
        } else {
            (filename, "")
        }
    };

    let mut i = 1;
    loop {
        let new_path =
            path_buf.with_file_name(format!("{path_buf_stem}_{}{path_buf_extension}", i));

        if !new_path.exists() {
            return new_path;
        }

        i += 1;
    }
}

pub fn generate_zip_package_filename(package_name: &str) -> String {
    let now = time::OffsetDateTime::now_local().unwrap_or_else(|_| time::OffsetDateTime::now_utc());

    let now_str = now
        .format(format_description!(
            "[year]-[month]-[day]_[hour].[minute].[second]"
        ))
        .expect("failed to format current time");

    format!("{package_name}_{now_str}.zip")
}
