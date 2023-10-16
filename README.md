## 环境变量说明

- `TEST_LOG` 运行测试的时候是否打印 tracing logs 默认为空
- `RUST_LOG` 可以控制 sqlx 的日志输出等级

## 命令运行 

注意要把命令行的代理关了，不然测试会有问题，可能和 wiremock 有关系

```bash

export RUST_LOG="sqlx=error,info"
export TEST_LOG=enabled
cargo t subscribe_fails_if_there_is_a_fatal_database_error | bunyan

```

## ch07

Let's go over the possible scenarios to convince ourselves that we cannot possibly deploy 
confirmation emails all at once without incurring downtime.

Transaction is a way to group multiple operations into one atomic operation.


## ch08

Orphan rule aside, it would still be a mistake for us to implement `ResponseError` for `sqlx:Error`.
We want to return a 500 Internal Server Error when we run into a `sqlx::Error` 
_while trying to persist a subscriber token_.
In another circumstance we might sish to handle `sqlx::Error` differently.

`Option<&(dyn Error + 'static)>`  
`dyn Error` is a trait object - a type that we know nothing about aport from the fact that it 
implements the Error trait.

函数 return 的 error 会隐式的调用 `.into()`?   
https://internals.rust-lang.org/t/what-is-wrong-with-auto-into/17319/2