mod proto;

use log::trace;
use sqlx::SqlitePool;
use tonic::{Request, Response, Result};

use self::proto::{
    data_storage_server::DataStorage, CreateRequest, CreateResponse, UpdateMetadataRequest,
    UpdateMetadataResponse,
};

pub use self::proto::data_storage_server::DataStorageServer;

#[derive(Debug)]
pub struct Storage {
    pub pool: SqlitePool,
}

#[tonic::async_trait]
impl DataStorage for Storage {
    async fn create(&self, request: Request<CreateRequest>) -> Result<Response<CreateResponse>> {
        trace!("create: {:?}", request);
        let metadata = request.into_inner().metadata;
        let name = metadata.as_ref().and_then(|x| x.name.as_ref());
        let description = metadata.as_ref().and_then(|x| x.description.as_ref());
        let tags = metadata.as_ref().map(|x| &x.tags);
        let id = self.create(name, description, tags).await;
        Ok(Response::new(CreateResponse { id }))
    }

    async fn update_metadata(
        &self,
        request: Request<UpdateMetadataRequest>,
    ) -> Result<Response<UpdateMetadataResponse>> {
        trace!("update_metadata: {:?}", request);
        let msg = request.into_inner();
        let id = msg.id;
        let metadata = msg.metadata;
        let name = metadata.as_ref().and_then(|x| x.name.as_deref());
        let description = metadata.as_ref().and_then(|x| x.description.as_deref());
        let tags = metadata.as_ref().map(|x| x.tags.as_slice());
        self.update(id, name, description, tags).await;
        Ok(Response::new(UpdateMetadataResponse {}))
    }
}

impl Storage {
    async fn create(
        &self,
        name: Option<&String>,
        description: Option<&String>,
        tags: Option<&Vec<String>>,
    ) -> i64 {
        let mut tx = self.pool.begin().await.unwrap();
        let id = sqlx::query!(
            "INSERT INTO data_indices (name, description) VALUES (?, ?) RETURNING id",
            name,
            description
        )
        .fetch_one(&mut *tx)
        .await
        .expect("insert data failed")
        .id;
        for tag in tags.into_iter().flatten() {
            let res = sqlx::query!("SELECT id FROM tags WHERE name = ?", tag)
                .fetch_optional(&mut *tx)
                .await
                .expect("query tag failed");
            let tag_id = match res {
                Some(r) => r.id,
                None => {
                    sqlx::query!("INSERT INTO tags (name) VALUES (?) RETURNING id", tag)
                        .fetch_one(&mut *tx)
                        .await
                        .expect("insert tag failed")
                        .id
                }
            };
            sqlx::query!(
                "INSERT INTO data_index_tag (data_index_id, tag_id) VALUES (?, ?)",
                id,
                tag_id
            )
            .execute(&mut *tx)
            .await
            .expect("insert relation failed");
        }
        tx.commit().await.expect("commit failed");
        id
    }

    async fn update(
        &self,
        id: i64,
        name: Option<&str>,
        description: Option<&str>,
        tags: Option<&[String]>,
    ) {
        let mut tx = self.pool.begin().await.unwrap();
        if let Some(name) = name {
            sqlx::query!("UPDATE data_indices SET name = ? WHERE id = ?", name, id)
                .execute(&mut *tx)
                .await
                .expect("update name failed");
        }
        if let Some(description) = description {
            sqlx::query!(
                "UPDATE data_indices SET description = ? WHERE id = ?",
                description,
                id
            )
            .execute(&mut *tx)
            .await
            .expect("update description failed");
        }
        if let Some(tags) = tags {
            sqlx::query!("DELETE FROM data_index_tag WHERE data_index_id = ?", id)
                .execute(&mut *tx)
                .await
                .expect("delete relation failed");
            for tag in tags {
                let res = sqlx::query!("SELECT id FROM tags WHERE name = ?", tag)
                    .fetch_optional(&mut *tx)
                    .await
                    .expect("query tag failed");
                let tag_id = match res {
                    Some(r) => r.id,
                    None => {
                        sqlx::query!("INSERT INTO tags (name) VALUES (?) RETURNING id", tag)
                            .fetch_one(&mut *tx)
                            .await
                            .expect("insert tag failed")
                            .id
                    }
                };
                sqlx::query!(
                    "INSERT INTO data_index_tag (data_index_id, tag_id) VALUES (?, ?)",
                    id,
                    tag_id
                )
                .execute(&mut *tx)
                .await
                .expect("insert relation failed");
            }
        }
        tx.commit().await.expect("commit failed");
    }
}
