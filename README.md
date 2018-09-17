# 目标
给出一个正则表达式，生成一段能够匹配对应字符串的 `rust` 代码。

## 生成正则描述的 NFA
```rust
let r: RegexItem = r#"a([b\d]?c|d)+"#.into();
let mut t = TransTable::from_nfa(&r.nfa_graph());
t.cut_epsilon();

println!("{}", t.to_dot_graph());
```

![regex-nfa](https://github.com/sbwtw/regex-gen/blob/master/graphviz.png)
