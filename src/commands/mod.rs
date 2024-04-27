use songbird::serenity::get as get_songbird;

pub type Data = (); // User data, which is stored and accessible in all command invocations
pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, Data, Error>;

mod join;
mod play;
mod seek;
mod skip;

pub use join::join;
pub use play::play;
pub use seek::seek;
pub use skip::skip;
