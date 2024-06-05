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
