use tonic::{Request, Response, Result, Status};
use tracing::warn;

use crate::{
    IPC_PROTOCOL_VERSION, VERSION,
    app::{AppError, AppHandle},
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
        Ok(Response::new(VersionResponse {
            app_version: VERSION.into(),
            protocol_version: IPC_PROTOCOL_VERSION,
        }))
    }

    async fn show_ui(&self, _request: Request<ShowUiRequest>) -> Result<Response<ShowUiResponse>> {
        self.app.request_show_ui().map_err(|err| match err {
            AppError::UiCommandUndelivered => {
                warn!("ShowUiRequest received but no desktop UI subscriber is attached");
                Status::failed_precondition("desktop UI is not attached to this workspace server")
            }
            AppError::StateDropped => {
                warn!("ShowUiRequest received while workspace server is shutting down");
                Status::unavailable("workspace server is shutting down")
            }
            _ => {
                warn!(error = %err, "ShowUiRequest failed with internal error");
                Status::internal("internal server error")
            }
        })?;
        Ok(Response::new(ShowUiResponse {}))
    }
}
