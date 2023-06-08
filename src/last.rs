macro_rules! last {
    (priv [$($_tts: tt $tag: tt)*] [$($tts: tt)*]) => { last!(priv {$($tag $tts)* }) };
    (priv { $(@ $tts: tt)* ^ $last: tt }) => { stringify!($last) };
    ($($tts: tt)*) => { last!(priv [$($tts @)* ^ ^] [@ $($tts)*]) }
}

macro_rules! zip {
    ($($v:expr),*;$($t:expr),*) => {
        [$(
            ($v,$t)
        ),*]
    };
}
#[test]
fn test() {
    println!("output={}", last!( a b c )); // output=c
    let zipped = zip!(1,2,3; "a","b","c");
}

// 作者：蟹妖
// 链接：https://www.zhihu.com/question/530509155/answer/3000698846
// 来源：知乎
// 著作权归作者所有。商业转载请联系作者获得授权，非商业转载请注明出处。


