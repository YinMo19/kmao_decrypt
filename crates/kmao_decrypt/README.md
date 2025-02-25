# KMAO_DECRYPTER
## YinMo19

顾名思义，这是一个解密“七猫免费小说”（以下简称七猫）下载到本地的小说的库。更多信息请参阅项目总文件夹的 [README](https://github.com/YinMo19/kmao_decrypt/)。这里是一些关于这个库的文档。

项目中使用的两个主要结构体是
```rs
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
```
两者都是从数据库查询获取的，一个是书本信息，一个是特定书本中选出的一章信息。简单的说，`Book` 中 `book_id` 是书本的唯一 id， 它实际上是一个约5-6位的数字，这个数字是在下载目录中的文件夹名字。而剩下的 `book_name` 和 `book_author` 是书本的名字和作者，这不必多说。而 `Chapter` 中 `chapter_index` 是章节的序号， `chapter_name` 是章节的名字， `chapter_id` 是章节的唯一 id，它作用也一样，是实际上存在的文件的名字。

也就是说，实际上在文件夹中存储的文件结构是
```
..../Documents/Book/<book_id>/<chapter_id>.txt
```
因此首先需要先获取书本的信息，然后获取章节的信息，然后才能解密。使用方法请参阅 `kmao_decrypt_bin`，一般情况下，只需要提供数据库位置和书本的 `Book`文件夹地址即可。调用 `select_chapters_from_sql` 选出需要的书本，然后调用 `chapters_join_sql` 解密并合并所需书本，最后将获取的信息通过 `write_to_book` 写入txt文件。

---

当没有提供数据库时，只是想要存粹解密整个文件夹中的内容，可以使用 `from_dir` 这个函数。它只会解算所有文件夹里面的文件而不做其他操作。当找不到数据库，可以尝试寻找缓存的章节信息 json，如果找到了的话，也可以考虑 `chapters_join` ，我也写了解析这个 json 的内容。

---
En Edition
---
As the name suggests, this is a library to decrypt novels downloaded locally from "Qimao Free Novels" (hereinafter referred to as Qimao). For more information, please refer to the [README](https://github.com/YinMo19/kmao_decrypt/) in the project's main folder. Here are some documents about this library.

The two main structs used in the project are:

```rust
/// Struct for chapter information.
/// Used for selecting chapters from the SQLite database.
#[derive(FromRow)]
pub struct Chapter {
    #[sqlx(rename = "ZCHAPTERINDEX")]
    chapter_index: i32,
    #[sqlx(rename = "ZCHAPTERNAME")]
    chapter_name: String,
    #[sqlx(rename = "ZCHAPTERID")]
    chapter_id: String,
}

/// Struct for book information.
#[derive(FromRow, Clone)]
pub struct Book {
    #[sqlx(rename = "ZBOOKID")]
    pub book_id: String,
    #[sqlx(rename = "ZBOOKNAME")]
    pub book_name: String,
    #[sqlx(rename = "ZBOOKAUTHOR")]
    pub book_author: String,
}
```

Both structs are obtained from database queries. One contains book information, and the other contains information about specific chapters within a book. In simple terms:

- Book contains `book_id`, which is the unique ID of the book. It is usually a 5-6 digit number that corresponds to the folder name in the download directory. The remaining fields `book_name` and `book_author` are the book's title and author.
- Chapter contains `chapter_index`, which is the chapter sequence number; `chapter_name`, which is the chapter title; and `chapter_id`, which is the unique ID of the chapter and also the actual file name.

Thus, the actual file structure stored in the folders is:
```
..../Documents/Book/<book_id>/<chapter_id>.txt
```

Therefore, you first need to obtain the book information, then get the chapter information, and finally decrypt it. For usage, please refer to `kmao_decrypt_bin`. Generally, you only need to provide the database location and the path to the book folder. Call select_chapters_from_sql to select the desired book, then call `chapters_join_sql` to decrypt and merge the selected book, and finally write the retrieved information to a .txt file using write_to_book.

When no database is provided and you just want to decrypt all contents in a folder, you can use the `from_dir` function. It will only decrypt all files in the folder without performing any other operations. If you cannot find the database, you can try looking for cached chapter information in a JSON file. If found, you can also consider using `chapters_join`, as I have written code to parse this JSON content.