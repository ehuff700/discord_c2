pub mod exploitation;
pub mod process;
pub mod recon;
pub mod utils;

#[macro_export]
macro_rules! say {
    ($ctx:expr, $fmt:expr $(, $arg:expr)*) => {
        if let Err(why) = $ctx.say(format!($fmt, $($arg),*)).await {
            tracing::error!("error sending discord message: {}", why);
        }
    }
}

#[macro_export]
macro_rules! reply {
    ($ctx:expr, $fmt:expr $(, $arg:expr)*) => {{
        let message = format!($fmt, $($arg),*);
        if let Err(why) = $ctx.send(poise::CreateReply::default().content(message).reply(true)).await {
            tracing::error!("error sending discord message: {}", why);
        }
    }}
}

#[macro_export]
macro_rules! reply_as_attachment {
	($ctx:expr, $buffer:expr) => {{
		if let Err(why) = $ctx
			.send(
				poise::CreateReply::default()
					.attachment(CreateAttachment::bytes($buffer, "message.txt"))
					.reply(true),
			)
			.await
		{
			tracing::error!("error sending discord message: {}", why);
		}
	}};
	($ctx:expr, $buffer:expr, $filename:expr) => {{
		use poise::serenity_prelude::*;
		if let Err(why) = $ctx
			.send(
				poise::CreateReply::default()
					.attachment(CreateAttachment::bytes($buffer, $filename))
					.reply(true),
			)
			.await
		{
			tracing::error!("error sending discord message: {}", why);
		}
	}};
}
