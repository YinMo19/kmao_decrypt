#![doc = include_str!("../README.md")]
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
    // check the all books from the database.
    // and select the book you want to decrypt.
    // (this function will show you a prompt)
    let selected_book = select_chapters_from_sql(
        &Path::new("Library/Application Support/com.yueyou.cyreader/QMNovel.sqlite"),
        Path::new("Documents/Book/"),
    )
    .await
    .expect("select failed.");

    // give the same two dir param as the select_chapters_from_sql
    // and the book you selected. And it will return a Vector of
    // (chapter_id, chapter_name, chapter_content).
    let novel_content = chapters_join_sql(
        &Path::new("Library/Application Support/com.yueyou.cyreader/QMNovel.sqlite"),
        Path::new("Documents/Book/"),
        &selected_book,
    )
    .await?;

    // write the novel content to a file.
    // it'll write the basic info, which read from database,
    // just like the book name, author.
    // and all the chapters sort by chapter id.
    let _ = write_to_book(&selected_book, novel_content)?;

    Ok(())
}
