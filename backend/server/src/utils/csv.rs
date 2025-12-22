use chrono::TimeDelta;
use futures::StreamExt;
use serde::Deserialize;
use serde::de::DeserializeOwned;
use tokio::io::AsyncRead;

#[derive(Deserialize)]
struct Row<T> {
    pub elapsed: i64,
    #[serde(flatten)]
    pub inner: T,
}

pub async fn read_csv<D: DeserializeOwned, R: AsyncRead + Send + Unpin>(
    reader: R,
) -> Result<Vec<(TimeDelta, D)>, csv_async::Error> {
    let mut csv_reader = csv_async::AsyncReaderBuilder::new()
        .has_headers(true)
        .create_deserializer(reader);

    let mut stream = csv_reader.deserialize::<Row<D>>();

    let mut result = vec![];
    while let Some(item) = stream.next().await {
        let row = item?;
        let elapsed = TimeDelta::milliseconds(row.elapsed);

        result.push((elapsed, row.inner));
    }

    Ok(result)
}
