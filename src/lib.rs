use base64::Engine;
use base64::engine::general_purpose;
use openssl::symm::{Cipher, Crypter, Mode};
use rayon::prelude::*;
use serde_json::Value;
use sqlx::prelude::FromRow;
use sqlx::{SqlitePool, sqlite::SqliteConnectOptions};
use std::error::Error;
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::Path;

#[derive(FromRow)]
pub struct Chapter {
    #[sqlx(rename = "ZBOOKID")]
    book_id: String,
    #[sqlx(rename = "ZCHAPTERINDEX")]
    chapter_index: i32,
    #[sqlx(rename = "ZCHAPTERNAME")]
    chapter_name: String,
    #[sqlx(rename = "ZCHAPTERID")]
    chapter_id: String,
}

#[derive(FromRow)]
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
        .expect("padding and finalize failed."); // 自动处理 PKCS#5 填充

    decrypted_data.truncate(count);
    Ok(decrypted_data)
}

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

pub fn decrypt_file(path: &Path, key: &[u8], iv: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
    let encrypted_data = read_file_to_string(path)?;
    let encrypted_bytes = general_purpose::STANDARD
        .decode(encrypted_data)
        .expect(&format!("base64 decode failed for {path:?}"));

    Ok(decrypt_aes_cbc(&encrypted_bytes, key, iv)?)
}

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

pub async fn chapters_join_sql(
    sql_path: &Path,
    source_path: &Path,
    novel: &Book,
) -> Result<(), Box<dyn Error>> {
    let options = SqliteConnectOptions::new().filename(sql_path);
    let pool = SqlitePool::connect_with(options).await?;

    let query = r#"select ZBOOKID, ZCHAPTERINDEX, ZCHAPTERNAME, ZCHAPTERID from ZCHAPTER"#;
    let chapters: Vec<Chapter> = sqlx::query_as(query).fetch_all(&pool).await?;

    let select_chapters = chapters
        .iter()
        .filter(|&chapter| chapter.book_id == novel.book_id)
        .collect::<Vec<&Chapter>>();

    // let mut novel_content = Vec::new();
    let key = b"242ccb8230d709e1";
    let iv = b"6443338373714701";

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

    novel_content.sort_unstable_by_key(|chapter| chapter.0);

    let novel_name = format!("{}.txt", novel.book_name);
    let mut novel_file = File::create(novel_name).unwrap();
    writeln!(novel_file, "{}\n{}\n", novel.book_name, novel.book_author)?;
    for (_, chapter_name, mut file_content) in novel_content {
        file_content = file_content.replace("\n", "\n\n");
        writeln!(novel_file, "{}\n\n{}\n\n", chapter_name, file_content)?;
    }

    Ok(())
}
