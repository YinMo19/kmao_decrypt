#![doc = include_str!("../README.md")]

use colored::*;
use kmao_decrypt::*;
use sqlx::{SqlitePool, sqlite::SqliteConnectOptions};
use std::{error::Error, path::Path};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // from_dir("1648011", "wf")?;

    let options = SqliteConnectOptions::new().filename(&Path::new(
        "Library/Application Support/com.yueyou.cyreader/QMNovel.sqlite",
    ));
    let pool = SqlitePool::connect_with(options).await?;

    let query = r#"select ZBOOKID, ZBOOKNAME, ZBOOKAUTHOR from ZBOOK"#;
    let all_books: Vec<Book> = sqlx::query_as(query).fetch_all(&pool).await?;

    let book_downloaded = Path::new("Documents/Book/")
        .read_dir()?
        .map(|entry| {
            entry
                .unwrap()
                .path()
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .to_string()
        })
        .collect::<Vec<String>>();

    let mut book_to_select = Vec::new();
    for book in &book_downloaded {
        if book.starts_with(".") {
            continue;
        }
        for recorded_book in &all_books {
            if &recorded_book.book_id == book {
                book_to_select.push(recorded_book);
                break;
            }
        }
    }
    for (seq, book) in book_to_select.iter().enumerate() {
        println!(
            "{}. {},{}",
            seq,
            book.book_name.green().bold(),
            book.book_author.blue().bold()
        );
    }
    print!(
        "{}\n> ",
        "Please select the book you want to decrypt:"
            .purple()
            .bold()
    );
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    let input_num = input
        .trim()
        .parse::<usize>()
        .expect("please input a number.");

    let selected_book = book_to_select[input_num];

    chapters_join_sql(
        &Path::new("Library/Application Support/com.yueyou.cyreader/QMNovel.sqlite"),
        Path::new("Documents/Book/"),
        selected_book,
    )
    .await?;
    println!("decrypt success.");
    Ok(())
}
