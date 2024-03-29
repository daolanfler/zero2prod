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
`dyn Error` is a trait object - a type that we know nothing about apart from the fact that it
implements the Error trait.

函数 return 的 error 会隐式的调用 `.into()`?  
https://internals.rust-lang.org/t/what-is-wrong-with-auto-into/17319/2

### thiserror

- `#[error(/* */)]` defines the Display representation of the enum variant it is applied to.
- `#[source]` is used to dentoe what should be returned as root cause in Error::source.
- `#[from]` automatically derives an implementation of **From** for the type it has been applied to
  into the top-level error type (e.g. `impl From<StoreTokenError> for SubscribeError {/* */}`). The
  field annotated with `#[from]` is also used as error source, saving us from having to use two
  annotations on the same field.

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

_CRUD_ (create, delete, read, update). need some form of **expiration** - sessions are meant to
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

Sessions can be used for more than authentication - e.g. to keep track of what have been added to
the basket when shopping in "guest" mode. This implies that a user might be associated to an
_anonymous_ session and, after they authenticate, to a _priviledged_ session. This can be leveraged
by attackers.  
Websites go to great lengths to prevent malicious actors from sniffing session tokens, leading to
another attach strategy - seed the user's browser with a **known** session token **before** they log
in, wait for authentication to happen and, boom, you are in!.  
[rotating the session token when the user logs in](https://cheatsheetseries.owasp.org/cheatsheets/Session_Management_Cheat_Sheet.html#renew-the-session-id-after-any-privilege-level-change)

A Typed Interface To Session  
It works when the state is very simple, but it quickly degrades into a mess if you have several
routes accessing the same data - how can you be sure that you updateded all of them when you want
to evolve the schema? How do we prevent a key typo from causing a production outage?

Tests can help, but we can use the type system to make the problem go away entirely. We will build
a strongly-typed API on top of `Session` to access and modify the state - no more string keys and
type casting in our request handlers.

Seed Users  
There is no user in the database and we do not have a sign up flow for admins - the implicit
expectation has been that the application owner would become the first admin of the newsletter
_somehow_!
The seed admin should then be able to invite more collaborators if they wish to do so. You could
implment this login-protected functionality as an exercise! Look at the subscription flow for
inspiration.

## Ch11 Fault-tolerant Workflows

To deliver a reliable service in the face of failure we will have to explore new concepts:
idempotency (幂等性), locking, queues and background jobs.

### Network I/O misbehave

- retry in process, by adding some logic around the `get_confirmed_subscribers` call;
- give up by returning an error to the user. The user can then decide if they want to retry or not

The first option makes our application more resilient (弹性的) to spurious (虚假的) failures.
Nonetheless, you can
only perform a finite number of retries; you will have to give up eventually.
Our implementation opts for the second strategy from the get-go. It might result in a few more 500s,
but it is not incompatible with our overaching (总体的，包罗万象的) objective.

bails out: 退出

You might recognize the struggle: we are dealing with a **workflow**, a combination of multiple
**subtasks**.  
We faced something similar in chapter 7 when we had to execute a sequence of SQL queries to create a
a new subscriber. Back then, we opted for an all-or-nothing semantics using SQL transactions:
nothing happens unless all queries succeed. Postmark's API does not provide any kind of transactional
semantics - each API call is its own unit of work, we have no way to link them together.

Given that this is all about understanding the caller's intent, there is no better strategy than
empowering the caller
themselves to tell us what they are trying to do. This is commonly accomplished using
**idempotency keys**.

The caller generates a unique identifier, the idempotency key, for every state-altering operation
they waht to perform. The idempotency key is attached to the outgoing request, usually as an HTTP
header (e.g Idempotency-Key)

**synchronization**: the second one should not be processed until the first one has completed.

- Reject the second request by returning a `409 conflict` status code back to the caller.
- Wait until the first request completes processing. Then return the same response back to the
  caller.

price to pay:
both the client and the server need to keep an open connection while spinning idle, waiting for
the other task to complete.

### Save Response

Why does `HttpResponse` need to be generic over the body type in the first place? Can't it just use
`Vec<u8>` or similar bytes container?
HTTP/1.1 supports another mechanism to transfer data - `Trasfer-Encoding: chunked`, also known as
**HTTP streaming**.

### Synchronization

In-memory locks (e.g. `tokio::sync::Mutex`) would work if all incomming requests were being served
by a single API instance. This is not our case: our API is replicated, therefore the two requests
might end up being processed by two different instances.

Our synchronization mechanism will have to live out-of-process - our database being the natural
candidate.

- is the first request completed, we want to return the saved response.
- if the first request is still ongoing, we want to **wait**.

Here we use transaction isolation  
Postgres isolation & lock levels:

> In effect, a `SELECT` query sees the snapshot of the database as of the instant the query begins
> to run
> `UPDATE`, `DELETE`, `SELECT FOR UPDATE` [...] will only find target that were committed as of the
> command start time. However, such a target row might have already been updated (or deleted or
> locked) by another concurrent transaction by the time it is found. In this case, **the would-be
> updater will wait for the first updating transaction to commit or roll back (if it is still in
> progress)**.

`could not serialize access due to concurrent update`  
`repeatable read` is designed to prevent non-repeatable reads (who would have guessed?): the same
`SELECT` query, if run twice in a row with the same transaction, should return the same data.  
This has consequences for statements such as `UPDATE`: if they are executed within a `repeatable read`
transaction, they cannot modify or lock rows changed by other transactions after the repeatable read  
transaction began.  
This is why the transaction initiated by the second request fails to commit in our little experiment
above. The same would have happened if we had chosen `serializable`, the strictest isolation level
available in Postgres.

### Distributed Transactions

The pain we are feeling is a common issue in real-world applications - you lose transactionality when
executing loogic that touches, at the same time, your local state and a remote state managed by
another system.

> More often then not, the other system lives within your organization - it's just a different
> microservice, with its own isolated data store. You have traded the inner complexity of the monolith
> for the complexity of orchestrating changes across multiple sub-system - complexity has to live
> somewhere.

**Backward Recovery**
**Foreward Recovery**

### Email Processing
We need to consume taks from `ussue_delivery_queue`.  
Multiple workers would pick the same task and we would end up with a lot of duplicated emails.
We need synchronization. Once again, we are going to leverage the database - we will use row-level
locks.  

Postgres 9.5 introduced the SKIP LOCKED clause - it allows SELECT statements to ignore all rows that 
are currently locked by another concurrent operation.  
FOR UPDATE, instead, can be used to lock the rows returned by a SELECT.

```sql
SELECT (newsletter_issue_id, subscriber_email)
FROM issue_delivery_queue
FOR UPDATE
SKIP LOCKED
LIMIT 1
```
This gives us a concurrency-safe queue.  
Each worker is going to select an uncontested task (SKIP LOCKED and LIMIT 1); the task itself is 
going to become unavailable to another workder (FOR UPDATE) for the duration of the over-arching SQL
transaction.  
When the task is complete (i.e. the email has been sent), we are going to delete the corresponding 
row from `issue_delivery_queue` and commit our changes.  