use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};

use crate::storage::api::object::{ObjectFolderList, ObjectResource};
use crate::storage::{Client, Error, Object};

/// Represents a Cloud Storage bucket.
#[derive(Clone)]
pub struct Bucket {
    pub(crate) client: Client,
    pub(crate) name: String,
}

impl Bucket {
    pub(crate) fn new(client: Client, name: impl Into<String>) -> Bucket {
        Bucket {
            client,
            name: name.into(),
        }
    }

    /// Get the bucket's name.
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    /// Insert a new object into the bucket.
    pub async fn create_object(
        &mut self,
        name: &str,
        data: impl Into<Vec<u8>>,
        mime_type: impl AsRef<str>,
    ) -> Result<Object, Error> {
        let client = &mut self.client;
        let inner = &client.client;
        let uri = format!(
            "{}/b/{}/o",
            Client::UPLOAD_ENDPOINT,
            utf8_percent_encode(&self.name, NON_ALPHANUMERIC),
        );

        let data = data.into();
        let token = client.token_manager.lock().await.token().await?;
        let request = inner
            .post(uri.as_str())
            .query(&[("uploadType", "media"), ("name", name)])
            .header("authorization", token)
            .header("content-type", mime_type.as_ref())
            .header("content-length", data.len())
            .body(data)
            .send();
        let response = request.await?;
        let string = response.error_for_status()?.text().await?;
        let resource = json::from_str::<ObjectResource>(string.as_str())?;

        Ok(Object::new(
            client.clone(),
            self.name.clone(),
            resource.name,
        ))
    }

    /// Get an object stored in the bucket.
    pub async fn object(&mut self, name: &str) -> Result<Object, Error> {
        let client = &mut self.client;
        let inner = &client.client;
        let uri = format!(
            "{}/b/{}/o/{}",
            Client::ENDPOINT,
            utf8_percent_encode(&self.name, NON_ALPHANUMERIC),
            utf8_percent_encode(name, NON_ALPHANUMERIC),
        );

        let token = client.token_manager.lock().await.token().await?;
        let request = inner
            .get(uri.as_str())
            .header("authorization", token)
            .send();
        let response = request.await?;
        let string = response.error_for_status()?.text().await?;
        let resource = json::from_str::<ObjectResource>(string.as_str())?;

        Ok(Object::new(
            client.clone(),
            self.name.clone(),
            resource.name,
        ))
    }

    /// Delete the bucket.
    pub async fn delete(self) -> Result<(), Error> {
        let client = self.client;
        let inner = client.client;
        let uri = format!(
            "{}/b/{}",
            Client::ENDPOINT,
            utf8_percent_encode(&self.name, NON_ALPHANUMERIC),
        );

        let token = client.token_manager.lock().await.token().await?;
        let request = inner
            .delete(uri.as_str())
            .header("authorization", token)
            .send();
        let response = request.await?;
        response.error_for_status()?;

        Ok(())
    }

    /// List all files within a folder.
    pub async fn list_files(&mut self, prefix: &str) -> Result<Vec<String>, Error> {
        let client = &mut self.client;
        let inner = &client.client;
        let uri = format!(
            "{}/b/{}/o",
            Client::ENDPOINT,
            utf8_percent_encode(&self.name, NON_ALPHANUMERIC),
        );

        let token = client.token_manager.lock().await.token().await?;
        let mut files = Vec::new();
        let mut page_token = String::from("");

        loop {
            let request = inner
                .get(uri.as_str())
                .header("authorization", &token)
                .query(&[
                    ("delimiter", "/"),
                    ("maxResults", "999"),
                    ("fields", "items/name,prefixes,nextPageToken"),
                    ("prefix", prefix),
                    ("pageToken", &page_token),
                ])
                .send();

            let response = request.await?;
            let string = response.error_for_status()?.text().await?;
            let resource = json::from_str::<ObjectFolderList>(string.as_str())?;

            if let Some(items) = resource.items {
                let mut list = items.into_iter().map(|e| e.name).collect::<Vec<String>>();
                files.append(&mut list);
            }

            if let Some(items) = resource.prefixes {
                let mut list = items.into_iter().map(|e| e.replace("/", "")).collect();
                files.append(&mut list);
            }

            if let Some(next_token) = resource.next_page_token {
                page_token = next_token.clone();
            } else {
                break;
            }
        }

        return Ok(files);
    }
}
