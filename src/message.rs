use std::collections::HashMap;


pub enum Message {
    Plain(String),
    Html(Html),
    Sticker {
        catagory: String,
        code: u16,
    }
}

pub struct Html {
    pub tag: String,
    pub attrs: HashMap<String, String>,
    pub children: Vec<Html>, 
}
macro_rules! html_children {
      
    () => {
        vec![]
    };
    ({
        |$($child: tt)*
    }) => {
        vec![
            $(html!($($child)*),)*
        ]
    }
}
macro_rules! html {
    (<$tag: ident $($attr: ident=$val:literal)* />) => {
        {
            let tag = stringify!($tag).to_lowercase().to_string();
            #[allow(unused_mut)]
            let mut attrs = HashMap::new();
            $(
                attrs.insert(stringify!($attr).to_lowercase().to_string(), $val.to_string());
            )*
            let html = Html {
                tag,
                attrs,
                children: vec![],
            };
            html
        }
    };
    (<$tag: ident $($attr: ident=$val:literal)* /> [
        $($child:expr),*
    ]) => {
        {
            let tag = stringify!($tag).to_lowercase().to_string();
            #[allow(unused_mut)]
            let mut attrs = HashMap::new();
            $(
                attrs.insert(stringify!($attr).to_lowercase().to_string(), $val.to_string());
            )*
            let html = Html {
                tag,
                attrs,
                children: vec![
                    $($child),*
                ],
            };
            html
        }
    };

}

pub struct HtmlBuilder {
    tag: String,
}
macro_rules! message {
    (@plain $m: expr) => {
        Message::Plain($m.into())
    };
    (@html $($html: tt)*) => {
        Message::Html(html!($($html)*))
    };
    (@sticker $cata:tt: $code:tt) => {
        Message::Sticker {
            catagory: stringify!($cata).into(),
            code: ($code as u16),
        }
    };
}

#[test]
fn test() {
    let msg = message!(@plain "hello");
    let msg = message!(@html <h1/>);
    let msg = message!(@html <h1 width=500 />);
    let msg = message!(@html <h1 width=500 color="black"/> [
        html!(<span width=450/> [
            html!(<button width=300/>),
            html!(<button width=300/>)
        ])
    ]);
    let html = html!(<h1 width=500 color="black"/>);
    assert_eq!(html.tag, "h1");
    assert_eq!(html.attrs.get("width"), Some(&"500".to_string()));
    assert_eq!(html.attrs.get("color"), Some(&"black".to_string()));
    assert_eq!(html.children.len(), 1);
    let span = &html.children[0];
    assert_eq!(span.tag, "span");
    assert_eq!(span.attrs.get("width"), Some(&"450".to_string()));
    assert_eq!(span.children.len(), 1);
    let button = &span.children[0];
    assert_eq!(button.tag, "button");
    assert_eq!(button.attrs.get("width"), Some(&"300".to_string()));
    assert_eq!(button.children.len(), 0);
    let msg = message!(@sticker system:12);
}