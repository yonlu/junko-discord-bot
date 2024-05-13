use serenity::model::prelude::Message;
use serenity::Result;

/// Checks that a message successfully sent; if not, then logs why to stdout.
pub fn check_msg(result: Result<Message>) {
    if let Err(why) = result {
        println!("Error sending message: {:?}", why);
    }
}
