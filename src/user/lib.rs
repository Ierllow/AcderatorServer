use crate::common::{AppError, Msgpack, ResponseHeader};
use crate::user::{UserDataResponse, UserScores};

pub(super) async fn user_data(scores: UserScores) -> Result<Msgpack<UserDataResponse>, AppError> {
    Ok(Msgpack(UserDataResponse {
        header: ResponseHeader {
            code: 0,
            master: None,
        },
        scores: scores.scores,
    }))
}
