use tonic::{Request, Response, Result};

use crate::{
    VERSION,
    proto::{VersionRequest, VersionResponse, fricon_service_server::FriconService},
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
