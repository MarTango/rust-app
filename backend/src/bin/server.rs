use async_std;
use std::env;

use backend::app;

#[async_std::main]
pub async fn main() -> Result<(), std::io::Error> {
    let app = app::app();

    let port = env::var("PORT").unwrap_or("8000".to_string());
    app.listen(format!("127.0.0.1:{}", port)).await?;

    Ok(())
}
