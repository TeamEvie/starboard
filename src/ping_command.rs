use twilight_model::http::interaction::{
    InteractionResponse, InteractionResponseData, InteractionResponseType,
};

use crate::{Ctx, CommandReturn};

pub async fn handle_event(ctx: Ctx<'_>) -> CommandReturn {

    let data = InteractionResponseData {
        content: Some(String::from("Pong!")),
        ..Default::default()
    };

    let response = InteractionResponse {
        kind: InteractionResponseType::ChannelMessageWithSource,
        data: Some(data),
    };

    ctx.http
        .interaction(ctx.id)
        .create_response(ctx.i.id, &ctx.i.token, &response)
        .exec()
        .await?;

    Ok(())
}