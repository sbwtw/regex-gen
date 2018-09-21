# 目标
- [x] 实现一个简单的正则引擎
- [ ] 给出一个正则表达式，生成一段能够匹配对应字符串的 `rust` 代码
- [ ] 给出一组正则描述，生成一个词法分析器(Lexer)。

## 支持的正则语法

### 基本语法
| 语法  | 描述                       |
|-------|----------------------------|
| (a\|b)| 匹配任意一个子表达式       |
| [ab]  | 匹配集合中任意一个字符     |
| [0-9] | 匹配字符 '0'~'9'           |
| [\^ab]| 匹配除集合中的其它字符     |

### 限定符
| 符号 | 描述                |
|------|---------------------|
|  *   | 匹配 0 或多次       |
|  +   | 匹配 1 或多次       |
|  ?   | 匹配 0 或 1 次      |

### 元字符
| 符号 | 描述                        |
|------|-----------------------------|
|  \d  | 匹配字符 '0'~'9'            |
|  .   | 匹配除 '\n' 以外的其它字符  |

## 示例

### 正则匹配
```rust
let r: RegexItem = r#"a\d+b"#.into();
let mut t = TransTable::from_nfa(&r.nfa_graph());
t.as_dfa();

let ee = ExecuteEngine::with_transtable(t);
assert_eq!(ee.exact_match("a"), false);
assert_eq!(ee.exact_match("ab"), false);
assert_eq!(ee.exact_match("aab"), false);
assert_eq!(ee.exact_match("a0"), false);
assert_eq!(ee.exact_match("a0b"), true);
assert_eq!(ee.exact_match("a0123456789b"), true);
```

### 生成正则描述的 DFA
```rust
let r: RegexItem = r#"a([b\d]?c|d)+"#.into();
let mut t = TransTable::from_nfa(&r.nfa_graph());
t.as_dfa();

println!("{}", t.to_dot_graph());
```

![regex-nfa](https://github.com/sbwtw/regex-gen/blob/master/graphviz.png)
