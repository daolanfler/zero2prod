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

## ch08 Error Handling

Orphan rule aside, it would still be a mistake for us to implement `ResponseError` for `sqlx:Error`.
We want to return a 500 Internal Server Error when we run into a `sqlx::Error`
_while trying to persist a subscriber token_.
In another circumstance we might sish to handle `sqlx::Error` differently.

`Option<&(dyn Error + 'static)>`  
`dyn Error` is a trait object - a type that we know nothing about aport from the fact that it
implements the Error trait.

函数 return 的 error 会隐式的调用 `.into()`?  
https://internals.rust-lang.org/t/what-is-wrong-with-auto-into/17319/2

### thiserror

- `#[error(/* */)]` defines the Display representation of the enum variant it is applied to.
- `#[source]` is used to dentoe what should be returned as root cause in Error::source.
- `#[from]` automatically derives an implementation of **From** for the type ithas been applied to
  into the top-level error type (e.g. `impl From<StoreTokenError> for SubscribeError {/* */}`). The
  field annotated with `#[from]` is also used as error source, saving us from having to use two
  annotations on the save field.

We do not want to expose the implementation details of the fallible routines that get mapped to
`Unexpected Error` by `subscribe` - it must be **opaque**.

### anyhow

The `context` method is performing double duties here:

- it converts the error returned by our methods into an `anyhow:Error`;
- it enriches it with additional context around the intentions of the caller.

### anyhow Or thiserror

> `anyhow` is for applications, `thiserror` is for libraries.
> It is not the right framing to discuss error handling.  
> You need to reason about **intent**.

## ch09 Naive Newsletter Dilivery

shortcomings of the naive approach:

1. **Security**  
   Our POST `/newsletters` endpoint is unprotected - anyone can fire a request toit and broadcast to
   our entire audience, unchecked.
2. **You Only Get One Shot**  
   As soon you hit POST `/newsletters`, your content goes out ot your entire mailing list. No chance
   to edit or review it in draft mode before giving the green light for publishing.
3. **Performance**  
   We are sending emails out one at a time.  
   We wait for the current one to be dispatched successfully before moving on to the next in line.
   This is not a massive issue if you have 10 or 20 subscribers, but it becomes noticeable shortly
   afterwards: latency is going to be horrible for newsletters with a sizeable audience.
4. **Fault Tolerance**  
   If we fail to dispatch one email we bubble up the error using `?` and return a
   `500 Internal Server Error` to the caller.  
   The remaining emails are never sent, nor we retry to dispatch the failed one.
5. **Retry Safety**
   Many things can go wrong when communicating over the network. What should a consumer of our API do
   if thery experience a timeout or a `500 Internal Server Error` when calling our service?  
   They cannot retry - thery risk sending the newsletter issue tiwce to the entire mailing list.

## ch10 Authentication

- Basic auth
- Session based
- OAuth 2.0
- OpenId Connect
- JWT

### Basic Auth

use Argon2 and PHC String Format with `PasswordHash` trait  
Do not break the async executor  
**cooperative scheduling**

Furthermore, keeping an audit trail with shared credentials is a nightmare. When something goes
wrong, it is impossible to detemine who did what: was it really me? Was it one of the twenty apps
I shared credentials with? Who takes responsibility?  
This is the textbook scenario for OAuth2 - the third-party never gets to see our username and
password. They receive an opaque access token from the authentication server which our PI knows how
to inspect to grant (or deny) access.

### Login Form

