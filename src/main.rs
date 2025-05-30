use anyhow::Result;
use tokio_util::compat::{FuturesAsyncWriteCompatExt, TokioAsyncWriteCompatExt};

const OUTPUT_ZIP: &str = "output.zip";
const HELLO_TXT: &str = "hello.txt";

const DEFAULT_UNIX_FILE_PERMISSIONS: u16 = 0o100644;

#[tokio::main]
async fn main() -> Result<()> {
    let output_zip = tokio::fs::File::create(OUTPUT_ZIP).await?;

    let mut zip = async_zip::tokio::write::ZipFileWriter::new(output_zip.compat_write());
    
    let entry_builder =
        async_zip::ZipEntryBuilder::new(HELLO_TXT.into(), async_zip::Compression::Deflate)
            .last_modification_date(chrono::Local::now().to_utc().into())
            .unix_permissions(DEFAULT_UNIX_FILE_PERMISSIONS);

    let mut writer = zip.write_entry_stream(entry_builder).await?.compat_write();
    let mut reader = get_input_reader().await?;
    let _result = tokio::io::copy(&mut reader, &mut writer).await?;
    writer.into_inner().close().await?;
    zip.close().await?;

    Ok(())
}

async fn get_input_reader() -> Result<impl tokio::io::AsyncRead> {
    let input_bytes = "00000000000\n".as_bytes();
    Ok(tokio::io::BufReader::new(input_bytes))
}
