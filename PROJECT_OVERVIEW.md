# Zero2Prod - Project Overview

## What is Zero2Prod?

Zero2Prod is a **production-ready newsletter subscription service** built in Rust that demonstrates enterprise-grade web application development. This project showcases how to build reliable, secure, and scalable web services using modern Rust ecosystem tools and best practices.

## System Architecture

```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│   Web Browser   │    │   Load Balancer  │    │   Application   │
│                 │◄──►│                  │◄──►│    Instances    │
│  (Users/Admins) │    │                  │    │                 │
└─────────────────┘    └──────────────────┘    └─────────────────┘
                                                        │
                       ┌─────────────────────────────────┼─────────────────────────────────┐
                       │                                 ▼                                 │
                       │                    ┌─────────────────────┐                       │
                       │                    │    PostgreSQL       │                       │
                       │                    │     Database        │                       │
                       │                    │                     │                       │
                       │                    └─────────────────────┘                       │
                       │                                                                   │
                       ▼                                                                   ▼
          ┌─────────────────────┐                                        ┌─────────────────────┐
          │    Redis Session    │                                        │   Email Service     │
          │       Store         │                                        │    (Postmark)      │
          │                     │                                        │                     │
          └─────────────────────┘                                        └─────────────────────┘
```

## Core Business Logic

### 1. Newsletter Subscription Flow
```
User Subscribes → Email Confirmation Sent → User Confirms → Added to Mailing List
```

**Detailed Process:**
1. User submits name and email via `/subscriptions` endpoint
2. System validates input and stores subscriber with "pending" status
3. Confirmation email with unique token sent to user
4. User clicks confirmation link
5. System validates token and updates subscriber status to "confirmed"
6. User now receives future newsletters

### 2. Newsletter Publishing Flow
```
Admin Login → Compose Newsletter → Publish → Background Processing → Email Delivery
```

**Detailed Process:**
1. Administrator authenticates via web interface
2. Admin composes newsletter content (HTML and text versions)
3. Newsletter published with idempotency key
4. System queues email delivery jobs for all confirmed subscribers
5. Background workers process email delivery queue
6. Emails sent via external email service (Postmark)

## Database Schema

### Core Tables
- **`subscriptions`** - Subscriber information and confirmation status
- **`subscription_tokens`** - Email confirmation tokens
- **`users`** - Administrator accounts with password hashes
- **`newsletter_issues`** - Published newsletter content
- **`issue_delivery_queue`** - Background job queue for email delivery
- **`idempotency`** - Idempotency key tracking for duplicate prevention

## Key Technical Features

### Authentication & Security
- **Multi-layered Authentication**: Basic Auth and Session-based authentication
- **Password Security**: Argon2 password hashing with PHC string format
- **Session Management**: Redis-backed sessions with secure cookies
- **CSRF Protection**: Message Authentication Codes (MAC) for form security
- **XSS Prevention**: HTML escaping and secure content handling

### Reliability & Fault Tolerance
- **Idempotent Operations**: Prevents duplicate newsletter sends
- **Database Transactions**: ACID compliance for data consistency
- **Retry Mechanisms**: Graceful handling of transient failures
- **Background Job Processing**: Asynchronous email delivery with queue management
- **Distributed Locking**: PostgreSQL row-level locks for multi-instance deployments

### Performance & Scalability
- **Async Processing**: Non-blocking I/O throughout the application
- **Connection Pooling**: Efficient database connection management
- **Background Workers**: Decoupled email processing from web requests
- **Structured Logging**: Comprehensive observability with tracing

## Development & Testing

### Testing Strategy
- **Unit Tests**: Individual component testing
- **Integration Tests**: API endpoint testing with test database
- **End-to-End Tests**: Complete workflow validation
- **Mock Services**: Email service mocking with WireMock

### Development Tools
- **Database Migrations**: SQLx migration management
- **Offline Query Checking**: Compile-time SQL validation
- **Docker Support**: Containerized deployment
- **CI/CD Ready**: GitHub Actions compatible

## Production Considerations

### Deployment
- **Docker Containerization**: Standardized deployment package
- **Environment Configuration**: Flexible configuration management
- **Health Checks**: Service monitoring endpoints
- **Graceful Shutdown**: Clean application termination

### Monitoring & Observability
- **Structured Logging**: JSON-formatted logs with tracing
- **Error Tracking**: Comprehensive error handling and reporting
- **Performance Metrics**: Request timing and throughput monitoring
- **Health Endpoints**: Service status verification

### Security Hardening
- **Input Validation**: Comprehensive data validation
- **SQL Injection Prevention**: Parameterized queries
- **Rate Limiting**: Protection against abuse
- **Secure Headers**: HTTP security headers implementation

## Educational Value

This project serves as a comprehensive reference for:
- **Rust Web Development**: Modern Rust web application patterns
- **Database Design**: Relational database modeling and optimization
- **Distributed Systems**: Handling consistency in distributed environments
- **Production Engineering**: Building reliable, maintainable systems
- **Security Practices**: Implementing security controls in web applications
- **Testing Methodologies**: Comprehensive testing strategies
- **DevOps Practices**: Deployment and operational considerations

## Technologies Used

### Core Stack
- **Rust**: Systems programming language
- **Actix-web**: High-performance web framework
- **SQLx**: Async SQL toolkit with compile-time checked queries
- **PostgreSQL**: Relational database
- **Redis**: In-memory data store for sessions
- **Tokio**: Async runtime

### Supporting Libraries
- **Serde**: Serialization/deserialization
- **Tracing**: Structured logging and instrumentation
- **Argon2**: Password hashing
- **Reqwest**: HTTP client for external API calls
- **UUID**: Unique identifier generation
- **Chrono**: Date and time handling

This project demonstrates how to build a production-ready web service that handles real-world requirements including security, reliability, performance, and maintainability.