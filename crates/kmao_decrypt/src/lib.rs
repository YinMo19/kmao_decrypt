#![doc = include_str!("../README.md")]

use base64::Engine;
use base64::engine::general_purpose;
use colored::*;
use openssl::symm::{Cipher, Crypter, Mode};
use rayon::prelude::*;
use serde_json::Value;
use sqlx::prelude::FromRow;
use sqlx::{SqlitePool, sqlite::SqliteConnectOptions};
use std::error::Error;
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::Path;

/// struct for used chapter info.
/// It used for select chapter from sqlite database.
#[derive(FromRow)]
pub struct Chapter {
    #[sqlx(rename = "ZCHAPTERINDEX")]
    chapter_index: i32,
    #[sqlx(rename = "ZCHAPTERNAME")]
    chapter_name: String,
    #[sqlx(rename = "ZCHAPTERID")]
    chapter_id: String,
}

/// The Book info.
#[derive(FromRow, Clone)]
pub struct Book {
    #[sqlx(rename = "ZBOOKID")]
    pub book_id: String,
    #[sqlx(rename = "ZBOOKNAME")]
    pub book_name: String,
    #[sqlx(rename = "ZBOOKAUTHOR")]
    pub book_author: String,
}

pub fn read_file_to_string<P: AsRef<Path>>(path: P) -> Result<String, io::Error> {
    let mut file = File::open(path).expect("file not found");
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .expect("failed to read file");
    Ok(contents)
}

pub fn write_bytes_to_file<P: AsRef<Path>>(path: P, data: &[u8]) -> Result<(), io::Error> {
    let mut file = File::create(path).expect("create file failed");
    file.write_all(data).expect("fail to write file");
    Ok(())
}

/// A simple AES-CBC decryption function.
pub fn decrypt_aes_cbc(
    data: &[u8],
    key: &[u8],
    iv: &[u8],
) -> Result<Vec<u8>, openssl::error::ErrorStack> {
    let cipher = Cipher::aes_128_cbc();
    let mut decrypter =
        Crypter::new(cipher, Mode::Decrypt, key, Some(iv)).expect("chipher init failed");
    let mut decrypted_data = vec![0; data.len() + cipher.block_size()];
    let mut count = decrypter
        .update(data, &mut decrypted_data)
        .expect("decrypter update failed");

    count += decrypter
        .finalize(&mut decrypted_data[count..])
        .expect("padding and finalize failed."); // default PKCS#5 padding

    decrypted_data.truncate(count);
    Ok(decrypted_data)
}

/// Decrypt all files in a directory.
pub fn from_dir(path: &str, out_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let key = b"242ccb8230d709e1";
    let iv = b"6443338373714701";
    let dir = Path::new(path);
    let out_dir = Path::new(out_path);

    // ensure out_dir exists
    if !out_dir.exists() {
        std::fs::create_dir_all(out_dir)?;
    }

    for entry in dir.read_dir()? {
        let entry = entry?;
        let path = entry.path();
        assert!(path.is_file() && path.extension().unwrap() == "txt");
        // println!("{path:?}");

        // if path's filename ends with "-tmp.txt", skip it
        if path
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .ends_with("-tmp.txt")
        {
            // println!("skip {path:?}");
            continue;
        }

        let decrypt_file_content = decrypt_file(&path, key, iv)?;
        let out_path = out_dir.join(path.file_name().unwrap());

        write_bytes_to_file(out_path, &decrypt_file_content[16..])?;
    }

    Ok(())
}

/// Decrypt a file using AES-CBC.
pub fn decrypt_file(path: &Path, key: &[u8], iv: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
    let encrypted_data = read_file_to_string(path)?;

    // firstly decode the base64 string.
    // the base64::decode function was deprecated,
    // so we use the general_purpose::STANDARD instead.
    let encrypted_bytes = general_purpose::STANDARD
        .decode(encrypted_data)
        .expect(&format!("base64 decode failed for {path:?}"));

    Ok(decrypt_aes_cbc(&encrypted_bytes, key, iv)?)
}

/// From the cached json file, which refered the chapter list,
/// and auto join the chapter content into one file.
pub fn chapters_join(
    chapter_list_path: &Path,
    source_path: &Path,
    novel_name: Option<String>,
) -> Result<(), Box<dyn Error>> {
    let content = fs::read_to_string(chapter_list_path)?;
    let chapter_lists: Value = serde_json::from_str(&content)?;
    let mut novel_content = Vec::new();
    let key = b"242ccb8230d709e1";
    let iv = b"6443338373714701";

    // parsing the json file.
    for chapter in chapter_lists["data"]["chapter_lists"].as_array().unwrap() {
        let chapter_id = chapter["id"].as_str().unwrap();
        let chapter_name = chapter["title"].as_str().unwrap();

        let file_path = source_path
            .join(chapter_lists["data"]["id"].as_str().unwrap())
            .join(format!("{}.txt", chapter_id));
        let file_content = decrypt_file(&file_path, key, iv)?;

        novel_content.push((
            chapter_name,
            String::from_utf8(file_content[16..].to_vec()).unwrap(),
        ));
    }

    let novel_name = format!(
        "{}.txt",
        novel_name.unwrap_or(chapter_lists["data"]["id"].as_str().unwrap().to_string())
    );
    let mut novel = File::create(novel_name).unwrap();
    for (chapter_name, mut file_content) in novel_content {
        file_content = file_content.replace("\n", "\n\n");
        writeln!(novel, "{}\n\n{}\n\n", chapter_name, file_content)?;
    }

    Ok(())
}

/// Same function as `chapters_join` but use sqlite database to get the chapter list.
/// It should make sure you have the sqlite database.
pub async fn chapters_join_sql(
    sql_path: &Path,
    source_path: &Path,
    novel: &Book,
) -> Result<Vec<(i32, String, String)>, Box<dyn Error>> {
    let options = SqliteConnectOptions::new().filename(sql_path);
    let pool = SqlitePool::connect_with(options).await?;

    // select the book's info chosen before.
    let query = r#"
        SELECT 
            ZCHAPTERINDEX, ZCHAPTERNAME, ZCHAPTERID 
        FROM 
            ZCHAPTER 
        WHERE 
            ZBOOKID = ?
    "#;
    let select_chapters: Vec<Chapter> = sqlx::query_as(query)
        .bind(&novel.book_id)
        .fetch_all(&pool)
        .await?;

    // just decrypt the file
    let key = b"242ccb8230d709e1";
    let iv = b"6443338373714701";

    // the novel content is split by chapter into so many files,
    // so we use par_iter to parallel the decryption process.
    // But accurately, the decryption process is not very fast(contrast with the normal method).
    let mut novel_content = select_chapters
        .into_par_iter()
        .map(|chapter| {
            let file_path = source_path
                .join(&novel.book_id)
                .join(format!("{}.txt", chapter.chapter_id));
            let file_content = decrypt_file(&file_path, key, iv).expect("decrypt_file_error");

            (
                chapter.chapter_index,
                chapter.chapter_name.clone(),
                String::from_utf8(file_content[16..].to_vec()).unwrap(),
            )
        })
        .collect::<Vec<_>>();

    // after the parallel process, we sort the novel_content by chapter_index.
    novel_content.sort_unstable_by_key(|chapter| chapter.0);
    Ok(novel_content)
}

/// Write the novel content to a file.
pub fn write_to_book(
    selected_book: &Book,
    novel_content: Vec<(i32, String, String)>,
) -> Result<(), Box<dyn Error>> {
    let novel_name = format!("{}.txt", selected_book.book_name);
    let mut novel_file = File::create(novel_name).unwrap();
    writeln!(
        novel_file,
        "{}\n{}\n",
        selected_book.book_name, selected_book.book_author
    )
    .expect("cannot write bookname and author.");
    for (_, chapter_name, mut file_content) in novel_content {
        // in almost the eBook, the lines should be parsed by \n\n.
        // this behavior is same as the markdown.
        file_content = file_content.replace("\n", "\n\n");
        writeln!(novel_file, "{}\n\n{}\n\n", chapter_name, file_content)
            .expect("cannot write bookname and author.");
    }

    Ok(())
}

/// Make prompt to select the book from the database.
/// It will check all the books you have downloaded.
pub async fn select_chapters_from_sql(
    database_path: &Path,
    book_path: &Path,
) -> Result<Book, Box<dyn Error>> {
    let options = SqliteConnectOptions::new().filename(database_path);
    let pool = SqlitePool::connect_with(options).await?;

    // Actually it should be use a `select ... from ... where IN (xxx, xxx, ...);`,
    // but the reality is the database is so small,
    // and useing rust filter instead of sql filter is just Ok.
    // If we just using the dir name of .../Documents/Book/<book_name> to select,
    // we should ensure the user have NOT adding some weird file in it.
    // just like MacOS users, they will add a .DS_Store in the dir
    // (if they use finder explore the dir).
    //
    // the tech we mention above has a example below.
    // ```rs
    // let query = format!(
    //     "SELECT * FROM books WHERE title IN ({})",
    //     chunk.iter().map(|_| "?").collect::<Vec<_>>().join(", ")
    // );
    // let mut query_builder = sqlx::query_as::<_, Book>(&query);
    // for title in chunk {
    //     query_builder = query_builder.bind(title);
    // }
    //
    // let books = query_builder.fetch_all(&conn).await?;
    // ```
    let query = r#"select ZBOOKID, ZBOOKNAME, ZBOOKAUTHOR from ZBOOK"#;
    let all_books: Vec<Book> = sqlx::query_as(query).fetch_all(&pool).await?;

    let book_downloaded = Path::new(book_path)
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

    let book_to_select: Vec<Book> = all_books
        .into_iter()
        .filter(|book| book_downloaded.contains(&book.book_id))
        .collect();

    Ok(book_to_select[get_prompt(&book_to_select)].clone())
}

/// Make prompts.
fn get_prompt(book_to_select: &Vec<Book>) -> usize {
    for (seq, book) in book_to_select.iter().enumerate() {
        println!(
            "{}. {},{}",
            seq,
            book.book_name.green().bold(),
            book.book_author.blue().bold()
        );
    }
    print!(
        "{}\n",
        "Please select the book you want to decrypt:"
            .purple()
            .bold()
    );
    let mut input = String::new();
    std::io::stdin()
        .read_line(&mut input)
        .expect("failed to read line");
    input
        .trim()
        .parse::<usize>()
        .expect("please input a number.")
}
