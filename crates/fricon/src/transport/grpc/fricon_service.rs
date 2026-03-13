use tonic::{Request, Response, Result, Status};
use tracing::warn;

use crate::{
    VERSION,
    app::{AppError, AppEvent, AppHandle},
    proto::{
        ShowUiRequest, ShowUiResponse, VersionRequest, VersionResponse,
        fricon_service_server::FriconService,
    },
};

pub(crate) struct Fricon {
    pub(crate) app: AppHandle,
}

#[tonic::async_trait]
impl FriconService for Fricon {
    async fn version(
        &self,
        _request: Request<VersionRequest>,
    ) -> Result<Response<VersionResponse>> {
        let version = VERSION.into();
        Ok(Response::new(VersionResponse { version }))
    }

    async fn show_ui(&self, _request: Request<ShowUiRequest>) -> Result<Response<ShowUiResponse>> {
        self.app
            .send_event(AppEvent::ShowUiRequest)
            .map_err(|err| match err {
                AppError::EventUndelivered => {
                    warn!("ShowUiRequest received but no desktop UI subscriber is attached");
                    Status::failed_precondition(
                        "desktop UI is not attached to this workspace server",
                    )
                }
                AppError::StateDropped => {
                    warn!("ShowUiRequest received while workspace server is shutting down");
                    Status::unavailable("workspace server is shutting down")
                }
            })?;
        Ok(Response::new(ShowUiResponse {}))
    }
}
