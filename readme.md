# 声明宏
rust 声明宏的作用主要是把一段代码通过语法制导翻译解释成另一段代码。

## 宏的基本印象
假如我们在实现一个聊天软件的客户端，现在有三种消息种类，分别是普通文本`Plain`，html文档`Html`，表情贴纸`Sticker`
```rust
pub enum Message {
    Plain(String),
    Html(Html),
    Sticker {
        catagory: String,
        code: u16,
    }
}

fn send(m: &Message) {
    println!("send {m}");
}
```

我们可能会通过构造器来构造消息
```rust
Message::Html(
    Html::builder().tag("main")
        .content(
            vec![
                Html::builder().tag("h1").content(
                    "hello"
                ).build().into(),
                Html::builder().tag("p").content(
                    ...
                ).build().into(),
            ]
        )
        .build()?
)
```

但是我们觉得每次都写这么大一串太麻烦了，于是我们希望有这么一个宏
```rust
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
fn test() {
    let msg = message!(@plain "hello");
    let msg = message!(@html <h1/>);
    let msg = message!(@html <h1 width=500/>);
    let msg = message!(@sticker system:12);
}
```

这个宏有三个模式，分别是
1. 匹配开头的`@plain`, 后面匹配为一个表达式(`expr`)
2. 匹配开头的`@html`, 后面匹配为任意个toten tree(`tt`)
3. 匹配开头的`@sticker`, 后面匹配为被分号(`:`)分隔的两个toten tree(`tt`)

进一步的，我们可能还希望用宏来写html，这样就不用使用builder来构造，于是我们可能有这样的宏：
```rust
pub struct Html {
    pub tag: String,
    pub attrs: HashMap<String, String>,
    pub children: Vec<Html>, 
}
macro_rules! html {
    (<$tag: ident $($attr: ident=$val:literal)*/>) => {
        {
            let tag = stringify!($tag).to_lowercase().to_string();
            #[allow(unused_mut)]
            let mut attrs = HashMap::new();
            $(
                attrs.insert(stringify!($attr).to_lowercase().to_string(), $val.to_string());
            )*
            Html {
                tag,
                attrs,
                children: vec![],
            }
        }
    };
}
```
这个宏目前只有一个pattern，它期待一个`<`作为开头，接下来是解析一个标识符(`ident`)，然后是任意组“标识符(`ident`) 等号(`=`) 字面量(`literal`)”，最后以一个`/>`结尾。
这样我们使用
```rust
let html = html!(<h1 width=500/>);
```
就以类似html的语法声明了一个dom节点

为了添加自节点，我们可以添加第二个partten
```rust
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
```
在第二个pattern我们在后面匹配一个方括号，方括号里面的是该节点的子元素
```rust
let msg = message!(@html <h1 width=500 color="black"/> [
    html!(<span width=450/> [
        html!(<button width=300/>),
        html!(<button width=300/>)
    ])
]);
```
## 详细用法
接下来我们通过几个例子详细说明宏的用法
### 例子一，step by step
这里我们声明一个创建btreemap的宏
```rust
// macro define a BtreeMap
macro_rules! map {
    ($K:ty => $V:ty; $($k:expr => $v:expr),*$(,)?) => {
        {
            let mut _map = std::collections::btree_map::BTreeMap::<$K, $V>::new();
            $(
                _map.insert($k.into(), $v.into());
            )*
            _map
        }
    };
    ($($k:expr => $v:expr),*$(,)?) => {
        {
            let mut _map = std::collections::btree_map::BTreeMap::new();
            $(
                _map.insert($k, $v);
            )*
            _map
        }
    };
}
```
这个宏可以这样使用
```rust
let map = map!{
    String => String;
    "a" => "b",
    "c" => "d"
};

assert_eq!(map.get("a"), Some(&"b".to_string()));
assert_eq!(map.get("c"), Some(&"d".to_string()));
let map = map!{
    "a" => "b",
    "c" => "d"
};
assert_eq!(map.get("a"), Some(&"b"));
assert_eq!(map.get("c"), Some(&"d"));
```

在这其中，派生宏`map`把一段代码
```rust
String => String;
"a" => "b",
"c" => "d"
```
解释（展开）成了另一段代码

```rust
{
    let mut _map = std::collections::btree_map::BTreeMap::<String, String>::new();
    _map.insert("a".into(), "b".into());
    _map.insert("c".into(), "d".into());
    _map
}
```

这个block表达式的值是一个btreemap。

可以看见派生宏主要做了一件事，就是把一段可以被我们自定义语法规则匹配上的AST（语法分析树）转换成另一棵符合rust语法的AST。

接下来解释这一段派生宏

宏的每一段分别是
```
(模式) => {展开式}
```
这里我们的宏有两段，我们首先来看第一段
#### 模式
第一段定义了pattern `$K:ty => $V:ty; $($k:expr => $v:expr),*$(,)?`

前五个字符：`$K:ty`，这里声明了一个占位符`$K`，我们加上doller符号`$`来表示它是一个占位符，同时指定了它所期望匹配到的token类型：`ty`，这里`ty`表示这个token可以被解析为rust中的一个`类型`。

所以，在这里诸如`u64`，`String`，`MyStruct<T1,T2,0>` 这样的都可以匹配上。
我们我们目前的输入的是`String`，并继续接下来的匹配。

接下来是`=>`， 这里表示期望接下来的字符是一个向右的双箭头符号，除此之外任何字符串都不能被匹配

我们目前的输入是`String =>`，那么匹配成功

接下来是`$V:ty`， 这里同`$K:ty`一样，表示接下来的输入仍然期望为一个rust类型，我们输入是`String => String`，匹配成功

接一下是一个分号`;`，我们输入一个分号，匹配上了。

接下来是`$($k:expr => $v:expr),*`，`$(...),*`是用来匹配重复的。这一段可以匹配`$k:expr => $v:expr`的多次重复，并以`,`分隔开，`*`指示我们可以匹配任意次。在`$k:expr => $v:expr`中，我们匹配了“表达式 右双箭头 表达式”

假如我们接下来输入的是
```rust
"a" => "b",
"c" => "d"
```
这里的字符串是一个字面量，自然也可以被解释成一个表达式，所以我们匹配到“表达式 右双箭头 表达式”

最后一段是`$(,)?`，这里我们匹配一个逗号零次或一次。这里我们没有输入，匹配到零次。

到这里，我们就已经完成匹配了。

```
    String => String;  "a" => "b", "c" => "d"
    $K:ty => $V:ty;    $($k:expr => $v:expr),   *$(,)?
```

#### 展开
匹配成功后，宏要做的就是展开
```rust
{
    let mut _map = std::collections::btree_map::BTreeMap::<$K, $V>::new();
    $(
        _map.insert($k.into(), $v.into());
    )*
    _map
}
```
展开的部分很好理解，基本上就是按照读取顺序，把占位符换成占位符对应上的`token`， 这里
```rust
    $(
        _map.insert($k.into(), $v.into());
    )*
```
将匹配到的重复多次的模式展开，没有连接符，并把每一段中占位符`$k`,`$v`对应替换
所以最后替换的结果就是
```rust
{
    let mut _map = std::collections::btree_map::BTreeMap::<String, String>::new();
    _map.insert("a".into(), "b".into());
    _map.insert("c".into(), "d".into());
    _map
}
```

#### 第二种模式
有时候我们并不想显示的指定类型，而是想让编译器去推导。于是我们提供第二种模式
`$($k:expr => $v:expr),*$(,)?`

#### 参考
每个segment可以有如下类型，具体可以参考rustbook，或者[这本](https://veykril.github.io/tlborm/decl-macros/minutiae/fragment-specifiers.html)
```
block
expr
ident
item
lifetime
literal
meta
pat
pat_param
path
stmt
tt
ty
vis
```
### 例子二 递归
宏是可以递归的，这里我们声明一个计算多个值中最大值的宏。
```rust
macro_rules! max {
    ($first:expr $(,)?) => {
        $first
    };
    ($first:expr, $($v:expr),*$(,)?) => {
        $first.max(max!($($v),*))
    };
}
#[test]
fn test() {
    let mut a = 1;
    let mut b = 2;
    let max = max!(a, b, a, b, a, b);
    let max = max!(a);
}
```

这里我们首先匹配括号里只有一个表达式的情况，如果匹配上，那就返回这个表达式

其次，当括号里有多个表达式时，我们取出匹配到的第一个表达式，调用第一个表达式的`max`方法，参数就是用`max`宏计算出的剩下的所有表达式中的最大值。

这样我们通过递归展开就实现了计算n个值中的最大值

其中`max`方法是`trait Ordering`提供的。


### 例子三 标识符与表达式
这里我们写一系列矩阵的宏，在这些宏，我们通过数学上“定义式”的方式来定义矩阵运算

```rust
#![feature(generic_arg_infer)]

#[derive(Debug, Clone, Copy)]
pub struct Matrix<T, const X:usize, const Y:usize> ([[T; X]; Y]);

macro_rules! mat {
    ($([$($e: expr),*$(,)?]),*$(,)?) => {
        $crate::mat::Matrix([
            $([$($e),*],)*
        ])
    }
}

macro_rules! mat_tranform {
    ($BX:expr, $BY: expr; $a:ident=$a_val:expr; $b:ident[$x:ident][$y:ident] := $def: expr) => {
        {
            use std::mem::*;
            let mut $b: [[_; $BX]; $BY] = unsafe {
                #[allow(clippy::uninit_assumed_init)]
                MaybeUninit::uninit().assume_init()
            };
            let mut $a = $a_val;
            for $x in 0..($BX) {
                for $y in 0..$BY {
                    swap(&mut $b[$x][$y], &mut ($def));
                }
            }
            forget($a);
            Matrix($b)
        }
    };
}


```

在这个宏中，`$a`和`$b`都是我们传入的标识符，定义式`$def`是我们传入的表达式，在这个表达式中我们可以顺利的访问`$a`和`$b`这两个标识符，因为他们都是处于宏的作用域内的。

我们可以通过这个宏来定义矩阵的变换
```rust
/// 定义转置
impl<T, const X:usize, const Y:usize> Matrix<T,X,Y> {
    pub fn transpose(self) -> Matrix<T,Y,X> {
        mat_tranform!(Y,X; a=self.0; b[x][y] := a[y][x])
    }
}

/// 定义翻转
impl<T, const X:usize, const Y:usize> Matrix<T,X,Y> {
    pub fn flip_x(self) -> Self {
        mat_tranform!(Y,X; a=self.0; b[x][y] := a[X-x][y])
    }
    pub fn flip_y(self) -> Self {
        mat_tranform!(Y,X; a=self.0; b[x][y] := a[x][Y-y])
    }
}

let m = mat!(
    [1,2],
    [4,5],
    [7,8]
).transpose().flip_x().flip_y();
```

类似的可以定义二元运算
```rust
macro_rules! mat_biop {
    ($BX:expr, $BY: expr; $a:ident=$a_val:expr; $b:ident=$b_val:expr; $c:ident[$x:ident][$y:ident] := $def: expr) => {
        {
            use std::mem::*;
            let mut $c: [[_; $BX]; $BY] = unsafe {
                #[allow(clippy::uninit_assumed_init)]
                MaybeUninit::uninit().assume_init()
            };
            let $a = $a_val;
            let $b = $a_val;
            for $x in 0..($BX) {
                for $y in 0..$BY {
                    swap(&mut $c[$x][$y], &mut ($def));
                }
            }
            Matrix($c)
        }
    };
}
/// 分量加法
impl <T, const X:usize, const Y:usize> std::ops::Add for Matrix<T, X, Y>
where T: std::ops::Add<Output=T> + Copy
{
    type Output = Matrix<T, X, Y>;
    fn add(self, rhs: Self) -> Self::Output {
        mat_biop!(X,Y; a=self.0; b=rhs.0; c[x][y] := a[x][y] + b[x][y])
    }
}
/// 矩阵乘法
impl<T, const X: usize, const Y: usize, const Z: usize> Mul<Matrix<T, Y, Z>> for Matrix<T, X, Y>
where
    T: Mul<Output = T> + Add<Output = T> + Copy + Default,
{
    type Output = Matrix<T, X, Z>;
    fn mul(self, rhs: Matrix<T, Y, Z>) -> Self::Output {
        mat_biop!(X,Z; a=self.0; b=rhs.0; c[x][z] := (0..Y).map(|y|a[x][y] * b[y][z]).fold(T::default(), T::add))
    }
}

let m = mat!(
    [1,2],
    [4,5],
    [7,8]
);
let n = (m+m)*m.transpose();
```

### 例子四 元信息，默认值
假设我们有一个应用，它分别有一些模块，各个模块都有配置，对于每个模块的配置我们都希望有一些通用的特性，比如实现某个trait，从环境变量加载，设置默认值等

我们希望这个宏有类似typescript的语法，也就是可以在字段后面接上一个`[= expr]`来定义默认值

这个宏为配置结构体实现了`ModuleConfig`和`Default`，并且可以通过接一个等号指定默认值，如果没有指定默认值，就会使用`Default::default()`得到的值

其中`$(#[$attr:meta])*`是匹配meta信息，就是诸如`#[cfg(...)]`这种的标记，值得注意的是注释也是一种`meta`，因为注释相当于
`#[doc(....)]`


```rust
trait ModuleConfig {
    fn load_from_env() -> Self;
}

#[inline]
fn parse_from_toml<C: Default + for<'a> Deserialize<'a>>(s: &str) -> C {
    toml::from_str(s).unwrap_or_default()
}

macro_rules! def_module_config {
    ($Config:ident {
        $(
            $(#[$attr:meta])*
            $field:ident:$type:ty $(= $default_value:expr)?
        ),* $(,)?
    }) => {
        #[derive(Serialize, Deserialize, Debug, Clone)]
        #[serde(default)]
        pub struct $Config {
            $(
                $(#[$attr])*
                pub $field: $type
            ),*
        }
        impl Default for $Config {
            fn default() -> Self {
                #[allow(unreachable_code)]
                $Config {
                    $($field: 'val_block: {
                        $(break 'val_block $default_value.into();)?
                        break 'val_block Default::default();
                    }),*
                }
            }
        }
        impl ModuleConfig for $Config {
            fn load_from_env() -> Self {
                $Config {
                    $($field: 
                        std::env::var(&format!(
                            "{}_{}",
                            stringify!($Config).to_uppercase(),
                            stringify!($field).to_uppercase()
                        )).as_deref().map(parse_from_toml::<$type>).unwrap_or_default()
                    ),*
                }
            }
        }
    };
}
```

然后就可以定义每个模块的配置
```rust
def_config_module! {
    PgConfig {
        // 如host字段，从环境变量加载时，就会去读取`PGCONFIG_HOST`
        host: String = "localhost",
        port: u16 = 5432_u16,
        db: String = "selene_bot",
        user: String = "postgres",
        password: String = "postgres",
    }
}

def_config_module! {
    InfluxConfig {
        url: String = "http://localhost:8086",
        database: String = "selene_bot"，
    }
}

def_config_module! {
    SdkConfig {
        // 不指定默认值，默认值就是空字符串
        app_id: String,
        token: String,
    }
}
```

### 例子五 获取最后一个元素
这个是从知乎上抄的，它可以提取一段序列的最后一个元素，其中利用了一些占位符（`@`,`^`）来支持rust宏不支持的匹配方式
https://www.zhihu.com/question/530509155

其中priv是一个保留关键字
```rust
macro_rules! last {
    (priv [$($_tts: tt $tag: tt)*] [$($tts: tt)*]) => { last!(priv {$($tag $tts)* }) };
    (priv { $(@ $tts: tt)* ^ $last: tt }) => { $last };
    ($($tts: tt)*) => { last!(priv [$($tts @)* ^ ^] [@ $($tts)*]) }
}
```
详细我们展开
```
last!(a b c) 

=> ）(匹配第三条规则)

last!(priv [a @ b @ c @ ^ ^] [@ a b c])

=> (匹配第一条规则)

last!(priv {@ @ @ a @ b ^ c})

=> (匹配第二条规则)

c
```

### 例子六
serde_json 的`json`宏使用了 TT Munchers 模式，用声明宏就完成了对json的解析
[源码](https://docs.rs/serde_json/latest/src/serde_json/macros.rs.html)

它之所以叫TT Munchers（token树吞噬器），就是因为它一个TokenTree一个TokenTree的解析，把已经解析的TokenTree标记为其他分段标识符（比如表达式，逗号，分号等等）

我们只看其中一部分（对数组的匹配），我删掉了原来的注释并且自己加上了注释

首先第一个`@array`是我们人为添加的标记，它表示后面的tokens将要被解析为一个array

```rust
// 如果是被一对方括号包含的tokens，加上标记array，进入匹配array的模式
([ $($tt:tt)+ ]) => {
    $crate::Value::Array(json_internal!(@array [] $($tt)+))
};

// 所有的tt都被解释为了表达式，完成匹配。
(@array [$($elems:expr,)*]) => {
    json_internal_vec![$($elems,)*]
};

// 同上，区别是尾部没有多余的逗号
(@array [$($elems:expr),*]) => {
    json_internal_vec![$($elems),*]
};

// 标记     已匹配的            这次匹配的  剩下的 
// @array   [$($elems:expr,)*]  null        $($rest:tt)*
// 已经匹配的用方括号括起来，这就好像一个语法制导的构造器，在剩余tt的左边始终是一个有效的array
// 每次匹配后要么可以产生一个有效值，要么是一个错误
// 这也得益于json是LL(1)文法
// 这次匹配到一个关键字null
(@array [$($elems:expr,)*] null $($rest:tt)*) => {
    // 我们把null变成一个表达式放进了左边的中括号内，继续解析剩下的
    json_internal!(@array [$($elems,)* json_internal!(null)] $($rest)*)
};

// 同上，匹配到关键字true
(@array [$($elems:expr,)*] true $($rest:tt)*) => {
    json_internal!(@array [$($elems,)* json_internal!(true)] $($rest)*)
};

// 同上，匹配到关键字false
(@array [$($elems:expr,)*] false $($rest:tt)*) => {
    json_internal!(@array [$($elems,)* json_internal!(false)] $($rest)*)
};

// 同上，匹配到一个数组
(@array [$($elems:expr,)*] [$($array:tt)*] $($rest:tt)*) => {
    json_internal!(@array [$($elems,)* json_internal!([$($array)*])] $($rest)*)
};

// 同上，匹配到一个map/object
(@array [$($elems:expr,)*] {$($map:tt)*} $($rest:tt)*) => {
    json_internal!(@array [$($elems,)* json_internal!({$($map)*})] $($rest)*)
};

// 匹配到一个表达式，跟随着一个逗号
(@array [$($elems:expr,)*] $next:expr, $($rest:tt)*) => {
    json_internal!(@array [$($elems,)* json_internal!($next),] $($rest)*)
};

// 匹配到最后一个元素
(@array [$($elems:expr,)*] $last:expr) => {
    json_internal!(@array [$($elems,)* json_internal!($last)])
};

// 匹配到一个逗号
(@array [$($elems:expr),*] , $($rest:tt)*) => {
    json_internal!(@array [$($elems,)*] $($rest)*)
};

// 以上匹配全部失败，标记为一个unexpected token，交由json_unexpected!这个宏处理
// json_unexpected会返回空（也就是0个token的tt），把整个进入匹配的tokens变成unexpected token
// 这样的unexpected token会向上传递，导致整个结构体匹配失败
(@array [$($elems:expr),*] $unexpected:tt $($rest:tt)*) => {
    json_unexpected!($unexpected)
};
```