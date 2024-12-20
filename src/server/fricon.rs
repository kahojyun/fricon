use tonic::{Request, Response, Result};

use crate::{
    proto::{fricon_service_server::FriconService, VersionRequest, VersionResponse},
    VERSION,
};

pub struct Fricon;

#[tonic::async_trait]
impl FriconService for Fricon {
    async fn version(
        &self,
        _request: Request<VersionRequest>,
    ) -> Result<tonic::Response<VersionResponse>> {
        let version = VERSION.into();
        Ok(Response::new(VersionResponse { version }))
    }
}
