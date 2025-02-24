/// A simple example of decrypt KMao Novel.
/// You should run this example in your macOS.
/// And you should put the `kmao_decrypt` in the
/// directory 
/// ```sh
/// ~/Library/Containers/E87206BF-44CA-4932-8DDB-D5C0E189A8C3/Data
/// ```
/// I'm not sure if your QiMao dir is `E87206BF-44CA-4932-8DDB-D5C0E189A8C3`
/// if you aren't sure of it, you can find it in Finder.


use kmao_decrypt::*;
use std::{error::Error, path::Path};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let selected_book = select_chapters_from_sql(
        &Path::new("Library/Application Support/com.yueyou.cyreader/QMNovel.sqlite"),
        Path::new("Documents/Book/"),
    )
    .await
    .expect("select failed.");

    let novel_content = chapters_join_sql(
        &Path::new("Library/Application Support/com.yueyou.cyreader/QMNovel.sqlite"),
        Path::new("Documents/Book/"),
        &selected_book,
    )
    .await?;

    let _ = write_to_book(&selected_book, novel_content)?;
    Ok(())
}
