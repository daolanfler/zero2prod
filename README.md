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
- `#[from]` automatically derives an implementation of **From** for the type it has been applied to
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

### XSS

This is known as a **cross-site scripting** (XSS) attack.  
The attacher injects HTML fragments or JavaScript snippets into a trusted website by exploiting
dynamic content built from untrusted sources - e.g. user inputs, query parameters, etc.  
From a user perspective, XSS attacks are particularly insidious - the URL matches the one you
wanted to visit, therefore you are likely to trust the displayed content.

**Message Authentication Codes**  
We need a mechanism to verify that the query parameters have been set by our API and that they
have not been altered by a third party.  
known as a message authentication - it guarantees that the message has not been modified in
transit (integrity) and it allows you to vefiy the identity of the sender
(**data origin authentication**).  
`hmac` stands for _hash message authentication code_.

Error messages hould be **ephemeral**, we should not put it in the query parameter of the
`LOCATION` response header, as browser will keep history of it.

### Cookie and Flash Message

What Is A Cookie?

> a small piece of data that a server sends to a user's web browser. The browser may store the
> cookie and send it back to the same server with later requests.

We can use cookies to implement the same strategy we tried with query parameters:

- The user enters invalid credentials and submits the form
- `POST /login` sets a cookie containing the error message and redirects the user back to `GET /login`
- The browser calls `GET /login`, including the values of the cookies currently set for the user;
- `Get /login`'s request handler checkes the cookies to see if there is an error message to be
  rendered (server side rendering);
- `Get /login` returns the HTML form to the caller and deletes the error message from cookie.

When it comes to durability, there are two type of cookies: **session cookies** and
**persistent cookies**. Sesion cookies are stored in memory - they are deleted when the session
ends (i.e. the browser is closed). Persistent cookies, instead, are saved to disk and will be
there when you re-open the browser.

**Cookie Security**

- Secure  
  We can benefit from an additional layer of defense by marking newly created cookies as `Secure`
  : this instructs browsers to only attach the cooke to requests transmitted over secure connections.

- Http-Only  
  to prevent client-side JavaScript from accessing the cookie.  
  We can mark newly created cookes as `Http-Only` to hide from client-side code - the browser will
  store them and attach them to outgoing requests as usual, but will not bee able the see them.
- User manipulate via developer console

We need multiple layer of defence. Message authentication code (MAC), A cookie value with an
HMAC tag attached is often referred to as a **singed cookie**.  
that leads us to **actix-web-flash-messages**

### Sessions

Session-based authentication is a strategy to avoid asking users to provide their password on every
single page. Users are asked to authenticate once, via a login form: if successful, the sever
generates a one-time secret - an authenticated session token.

> Naming can be quite confusing - we are using the terms _session token/session cookie_ to refer to
> the client-side cookie associated to a **user** session. Later in this chapter, we will talk about
> the lifecycle of cookies, where _session cookie_ refers to a cookie whos life time is tied to a
> **browser** session.

We need a **session store** - the server must remember the tokens it has generated in order to
authorize future requests for logged-in users. We also want to asscociate information to each
active session - this is known as a **session state**.

- creation 
- retrieval 
- update 
- deletion 

*CRUD* (create, delete, read, update). need some form of **expiration** - sessions are meant to 
be short-lived. 

#### Redis  

Therefore, we can use the session token as key while the value is the JSON representation of the
session state - the application takes care of serialization/deserialization. 

What does `Session::insert` actually do, though? 
All operations performed against `Session` are executed in memory - they do not affect the state
of the session as seen by the storage backend. After the handler returns a response,
`SessionMiddleware` will inspect the in-importy state of `Session` - if it changed, it will call
Redis to update (or create) the state. It will also take care of setting a session cookie on the 
client, if there wasn't one already.