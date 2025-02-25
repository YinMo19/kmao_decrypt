# KMAO_DECRYPTER
## YinMo19

顾名思义，这是一个解密“七猫免费小说”（以下简称七猫）下载到本地的小说的程序。七猫下载到本地的文件夹（在mac上可以直接跑原生的ios（ipa）应用，我在mac上进行测试）中，有一个数据库文件用来存储下载到本地的小说信息，包括书名/作者/章节名等等。因此需要调用 `sqlx` 库对数据库进行解析和处理。

另外，加密形式为一次 aes/cbc 加密和一次 base64 加密，因此需要对加密后的字符串进行 base64 解码，再进行 aes/cbc 解密。iv 和 key 都是从二进制中逆向获取的，因此不保证不改变。

最核心的部分应该是
```rs
    chapters_join_sql(
        &Path::new("Library/Application Support/com.yueyou.cyreader/QMNovel.sqlite"),
        Path::new("Documents/Book/"),
        selected_book,
    )
```
这个函数传入三个参数，分别是所有小说的那个数据库（用于获取需要解密的小说章节信息），小说具体存放的路径（只需要到各个小说文件夹的上一级），以及选中的小说（是一个我定义好的 Book 结构体）。这个函数会直接在程序执行的目录下生成解密之后的小说。

另外还有一个 `chapters_join` 函数，这个是使用缓存的信息（不推荐使用）。每本书在缓存时会有一个 json 存放了整个目录信息，因此使用它也可以完成解密/拼接效果。但是缓存随时可能会被清掉，因此还是推荐使用 `chapters_join_sql` 函数。

---

关于七猫的文件存储目录，在各个平台下可能有所不同，这里我简单描述 mac 下的行为。在 mac 下，直接从 App Store 下载的七猫应用文件存储放在用户的 Library 目录中。具体路径（至少在我的电脑上）是
```
~/Library/Containers/E87206BF-44CA-4932-8DDB-D5C0E189A8C3
```
在这个目录下只有一个 `Data`，此下就是一个很接近真实用户目录的文件夹。
```sh
~/Library/Containers/E87206BF-44CA-4932-8DDB-D5C0E189A8C3/Data
> ls
Desktop      Downloads    Movies       Pictures     SystemData   tmp
Documents    Library      Music        StoreKit     
```
其中大部分文件夹都是软链接到真实用户的主文件夹中。例如
```sh
> l 
total 7688
drwx------@ 14 yinmo19  staff   448B Feb 21 00:16 .
drwx------@  5 yinmo19  staff   160B Feb 20 02:14 ..
lrwxr-xr-x   1 yinmo19  staff    31B Feb 20 02:09 .CFUserTextEncoding -> ../../../../.CFUserTextEncoding
lrwxr-xr-x   1 yinmo19  staff    19B Feb 20 02:09 Desktop -> ../../../../Desktop
drwx------  15 yinmo19  staff   480B Feb 21 00:44 Documents
lrwxr-xr-x   1 yinmo19  staff    21B Feb 20 02:09 Downloads -> ../../../../Downloads
drwx------  64 yinmo19  staff   2.0K Feb 21 00:44 Library
lrwxr-xr-x   1 yinmo19  staff    18B Feb 20 02:09 Movies -> ../../../../Movies
lrwxr-xr-x   1 yinmo19  staff    17B Feb 20 02:09 Music -> ../../../../Music
lrwxr-xr-x   1 yinmo19  staff    20B Feb 20 02:09 Pictures -> ../../../../Pictures
drwx------   3 yinmo19  staff    96B Feb 20 02:09 StoreKit
drwx------   2 yinmo19  staff    64B Feb 20 02:09 SystemData
...
```
而小说下载存放的地址为 `Documents/Book/`，也就是上文 demo 中调用 `chapters_join_sql` 函数时传入的第二个参数。而数据库文件存放的地址自然就是 `Library/Application Support/com.yueyou.cyreader/QMNovel.sqlite`。这个数据库文件中存放有 有声小说和普通小说的信息，很全。并且没有任何加密，因此只需要简单的 `sqlite3` 就可以查看其中的信息。这里我推荐一个使用 `flask` 写的小程序 `sqlite_web`，可以用 web 页面浏览和查询数据库。

因此如果你构建编译好了我的程序之后将可执行文件放到上述文件夹中(`.../Data/`下)，执行可执行文件中就会从你已经下载的文件里询问你要解密那本小说。输入你的选项（标号）稍等就可以解密了。

我的程序做了并行，但是和普通的写似乎没什么巨大提升。不过解密时间也不久，所以就没想着优化了。

如果觉得不错，可以给我点个星星。~~如果有时间我可能会写一篇 blog 来描述逆向和这个程序的细节，可以关注 `blog.yinmo19.top`~~ 我没时间。

---
En Edition
---
As the name suggests, this is a program to decrypt novels downloaded locally from "Qimao Free Novels" (hereinafter referred to as Qimao). The downloaded files on macOS can be accessed directly by running the native iOS (IPA) application, which I tested on my Mac. The folder contains a database file that stores information about the downloaded novels, including book titles, authors, chapter names, etc. Therefore, the sqlx library is used to parse and process this database.

Additionally, the encryption format involves AES/CBC encryption followed by Base64 encoding. Thus, the encrypted string needs to be Base64-decoded first, then AES/CBC decrypted. The IV and key are obtained by reverse engineering the binary, so they may change in the future.

The most critical part of the program is the following function call:

```rust
chapters_join_sql(
    &Path::new("Library/Application Support/com.yueyou.cyreader/QMNovel.sqlite"),
    Path::new("Documents/Book/"),
    selected_book,
)
```
This function takes three parameters:

- The database containing all novel information (used to retrieve the chapter information of the novel to be decrypted),
- The path where the novels are stored (only up to the parent directory of each novel's folder),
- And the selected novel (an instance of the Book struct defined in the code).

This function will generate the decrypted novel in the current working directory when executed.

There is also a `chapters_join` function, which uses cached information (not recommended). Each book has a JSON file storing the entire directory structure, allowing for decryption and concatenation. However, since the cache can be cleared at any time, it is recommended to use the `chapters_join_sql` function.

Regarding the storage directories for Qimao files, they may differ across platforms. Here, I describe the behavior on macOS. On macOS, the files downloaded via the App Store are stored in the user's Library directory. The specific path (at least on my computer) is:
```
~/Library/Containers/E87206BF-44CA-4932-8DDB-D5C0E189A8C3
```
Under this directory, there is only one Data folder, which resembles a typical user home directory:

```sh
~/Library/Containers/E87206BF-44CA-4932-8DDB-D5C0E189A8C3/Data
> ls
Desktop      Downloads    Movies       Pictures     SystemData   tmp
Documents    Library      Music        StoreKit     
```
Most folders are symbolic links to the actual user home directories. For example:

```sh
> l 
total 7688
drwx------@ 14 yinmo19  staff   448B Feb 21 00:16 .
drwx------@  5 yinmo19  staff   160B Feb 20 02:14 ..
lrwxr-xr-x   1 yinmo19  staff    31B Feb 20 02:09 .CFUserTextEncoding -> ../../../../.CFUserTextEncoding
lrwxr-xr-x   1 yinmo19  staff    19B Feb 20 02:09 Desktop -> ../../../../Desktop
drwx------  15 yinmo19  staff   480B Feb 21 00:44 Documents
lrwxr-xr-x   1 yinmo19  staff    21B Feb 20 02:09 Downloads -> ../../../../Downloads
drwx------  64 yinmo19  staff   2.0K Feb 21 00:44 Library
lrwxr-xr-x   1 yinmo19  staff    18B Feb 20 02:09 Movies -> ../../../../Movies
lrwxr-xr-x   1 yinmo19  staff    17B Feb 20 02:09 Music -> ../../../../Music
lrwxr-xr-x   1 yinmo19  staff    20B Feb 20 02:09 Pictures -> ../../../../Pictures
drwx------   3 yinmo19  staff    96B Feb 20 02:09 StoreKit
drwx------   2 yinmo19  staff    64B Feb 20 02:09 SystemData
...
```
The location where novels are stored is `Documents/Book/`, which is the second parameter passed to the `chapters_join_sql` function in the demo. The database file is located at `Library/Application Support/com.yueyou.cyreader/QMNovel.sqlite`. This database contains information about both audiobooks and regular novels and is not encrypted, so it can be viewed using simple tools like `sqlite3`. I recommend a small Flask-based web app called `sqlite_web` for browsing and querying the database through a web interface.

Therefore, if you compile and place the executable in the above directory (`.../Data/`), running the executable will prompt you to select which novel you want to decrypt from your already downloaded files. After entering your choice (the number), the program will decrypt the selected novel shortly.

The program uses parallel processing, but it doesn't seem to provide significant performance improvements over a single-threaded approach. However, the decryption time is short, so further optimization wasn't prioritized.

If you find this useful, please give me a star. ~~If I have time, I might write a blog post detailing the reverse engineering and development of this program; you can follow `blog.yinmo19.top` ~~Unfortunately, I don't have the time.