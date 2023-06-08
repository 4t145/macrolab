macro_rules! message {
    (@plain $m: expr) => {
        Message::Plain($m)
    };
    (@html $html: tt) => {
        Message::Html(html!($m))
    };
    (@markdown $html: tt) => {
        Message::Markdown(markdown!($m))
    };
    (@emoji $cata:expr => $code:expr) => {
        Message::Emoji {
            catagory: $cata,
            code: $code,
        }
    };
}