# async_zip_tokio_example

The projects that reproduces the issue when the archive created by `async_zip` fails the zip bomb detection when trying to unpack with `unzip`.

## Steps to reproduce the issue

Running the project creates `output.zip` file.

```
$ cargo run
   Compiling async_zip_tokio_example v0.1.0 (/home/dbushev/projects/luna/async_zip_tokio_example)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.74s
     Running `target/debug/async_zip_tokio_example`
```

Unzip the archive.

```
$ unzip -o output.zip
Archive:  output.zip
  inflating: hello.txt
error: invalid zip file with overlapped components (possible zip bomb)
 To unzip the file anyway, rerun the command with UNZIP_DISABLE_ZIPBOMB_DETECTION=TRUE environmnent variable
```

## Workaround

I assume the issue is with the zip64 extra field written by default (`bX` in the `zipinfo` output). Below are the differences between `output.zip` created with `async_zip` and `hello.zip` created with `zip` command line tool.

```
$ zipinfo output.zip
Archive:  output.zip
Zip file size: 269 bytes, number of entries: 1
-rw-r--r--  6.3 unx       12 bX defN 25-May-30 10:52 hello.txt
1 file, 12 bytes uncompressed, 13 bytes compressed:  -8.3%
```

```
$ zip hello.zip hello.txt
  adding: hello.txt (deflated 50%)
$ zipinfo hello.zip
Archive:  hello.zip
Zip file size: 174 bytes, number of entries: 1
-rw-r--r--  3.0 unx       12 tx defN 25-May-30 10:52 hello.txt
1 file, 12 bytes uncompressed, 6 bytes compressed:  50.0%
```

The workaround is to disable zip64 support in `ZipFileWriter`,

```diff
--- a/src/main.rs
+++ b/src/main.rs
@@ -10,7 +10,7 @@ const DEFAULT_UNIX_FILE_PERMISSIONS: u16 = 0o100644;
 async fn main() -> Result<()> {
     let output_zip = tokio::fs::File::create(OUTPUT_ZIP).await?;

-    let mut zip = async_zip::tokio::write::ZipFileWriter::new(output_zip.compat_write());
+    let mut zip = async_zip::tokio::write::ZipFileWriter::new(output_zip.compat_write()).force_no_zip64();

     let entry_builder =
         async_zip::ZipEntryBuilder::new(HELLO_TXT.into(), async_zip::Compression::Deflate)
```

this way the produced archive lacks an extra field (`bl`) and accepted by `unzip`.

```
$ zipinfo output.zip
Archive: output.zip
Zip file size: 145 bytes, number of entries: 1
-rw-r--r-- 6.3 unx 13 bl defN 25-May-30 11:06 hello.txt
1 file, 13 bytes uncompressed, 12 bytes compressed: 7.7%
```
