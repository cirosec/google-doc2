use google_doc2::GoogleDocsC2;

#[async_std::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let c2 = GoogleDocsC2::new(env!(
        "DOCS_URL",
        "Must specify a DOCS_URL in the environment variable!"
    ))
    .await?;

    c2.add_comment("Hello, World!").await?;

    Ok(())
}
