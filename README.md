# Google DoC2

This is the companion repository to [the blog post about Google DoC2](https://cirosec.de/en/news/google-doc2). Check out the blog post for details.

Note: This repository is provided only for the purposes of demonstrating the techniques in the blog post, there will be no further commits or other forms of maintenance in this repository.

## Shell Demo

1. Create a [new Goole Doc](https://docs.new)
2. Type some text in the document (any text will do)
3. Share the document using the "Anyone with the link can edit" option
4. Run the server:
```bash
$ export DOCS_URL="https://docs.google.com/document/d/XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX/edit?usp=sharing"
$ cargo run --example shell_server
```
6. Open a new terminal and run the agent:
```bash
$ export DOCS_URL="https://docs.google.com/document/d/XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX/edit?usp=sharing"
$ cargo run --example shell_agent
```
7. Open the document in a web browser and watch the magic happen as you use the server to execute commands in the agent.

## Using as a library

You can use this library to create your own custom agents and servers by using the `google-doc2` crate.

For example, to add a comment to a Google Doc, read out all the comments and then clear them, you can use the following code:

```rust
use google_doc2::GoogleDocsC2;

#[async_std::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let c2 = GoogleDocsC2::new(env!(
        "DOCS_URL",
        "Must specify a DOCS_URL in the environment variable!"
    ))
    .await?;

    c2.add_comment("Hello, World!").await?;
    dbg!(c2.read_all_comments().await?);
    c2.clear_comments().await?;

    Ok(())
}
```
