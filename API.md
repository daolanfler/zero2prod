# Zero2Prod API Documentation

## API Endpoints

### Health Check
**GET `/health_check`**
- **Purpose**: Service health monitoring
- **Authentication**: None required
- **Response**: `200 OK` if service is healthy

### Newsletter Subscription

**POST `/subscriptions`**
- **Purpose**: Subscribe to newsletter
- **Authentication**: None required
- **Content-Type**: `application/x-www-form-urlencoded`
- **Body Parameters**:
  - `name` (string, required): Subscriber's name
  - `email` (string, required): Valid email address
- **Responses**:
  - `200 OK`: Subscription initiated, confirmation email sent
  - `400 Bad Request`: Invalid input data
  - `500 Internal Server Error`: Server error

**Example Request**:
```bash
curl -X POST http://localhost:8080/subscriptions \
  -H "Content-Type: application/x-www-form-urlencoded" \
  -d "name=John%20Doe&email=john@example.com"
```

### Email Confirmation

**GET `/subscriptions/confirm`**
- **Purpose**: Confirm email subscription
- **Authentication**: None required (token-based)
- **Query Parameters**:
  - `subscription_token` (string, required): Unique confirmation token from email
- **Responses**:
  - `200 OK`: Subscription confirmed successfully
  - `400 Bad Request`: Invalid or expired token
  - `500 Internal Server Error`: Server error

**Example Request**:
```bash
curl "http://localhost:8080/subscriptions/confirm?subscription_token=abc123def456"
```

### Admin Authentication

**GET `/login`**
- **Purpose**: Display login form
- **Authentication**: None required
- **Response**: HTML login form

**POST `/login`**
- **Purpose**: Authenticate administrator
- **Authentication**: None required (login endpoint)
- **Content-Type**: `application/x-www-form-urlencoded`
- **Body Parameters**:
  - `username` (string, required): Admin username
  - `password` (string, required): Admin password
- **Responses**:
  - `303 See Other`: Successful login, redirect to dashboard
  - `400 Bad Request`: Invalid credentials
  - `500 Internal Server Error`: Server error

**POST `/logout`**
- **Purpose**: Log out administrator
- **Authentication**: Required (valid session)
- **Response**: `303 See Other` redirect to login page

### Admin Dashboard

**GET `/admin/dashboard`**
- **Purpose**: Admin management interface
- **Authentication**: Required (valid session)
- **Response**: HTML dashboard interface

### Newsletter Publishing

**GET `/admin/newsletters`**
- **Purpose**: Display newsletter composition form
- **Authentication**: Required (valid session)
- **Response**: HTML newsletter composition form

**POST `/admin/newsletters`**
- **Purpose**: Publish newsletter to confirmed subscribers
- **Authentication**: Required (valid session)
- **Content-Type**: `application/x-www-form-urlencoded`
- **Headers**:
  - `Idempotency-Key` (optional): Unique key to prevent duplicate sends
- **Body Parameters**:
  - `title` (string, required): Newsletter title
  - `html_content` (string, required): HTML version of newsletter
  - `text_content` (string, required): Plain text version of newsletter
- **Responses**:
  - `303 See Other`: Newsletter queued for delivery
  - `400 Bad Request`: Invalid input data
  - `401 Unauthorized`: Authentication required
  - `500 Internal Server Error`: Server error

**Example Request**:
```bash
curl -X POST http://localhost:8080/admin/newsletters \
  -H "Content-Type: application/x-www-form-urlencoded" \
  -H "Cookie: session=<session_cookie>" \
  -H "Idempotency-Key: unique-key-123" \
  -d "title=Weekly%20Update&html_content=<h1>Newsletter</h1>&text_content=Newsletter"
```

## Authentication

### Session-Based Authentication
- Admin endpoints require active session
- Sessions stored in Redis with configurable expiration
- Session cookies are secure and HTTP-only
- Automatic session renewal on activity

### Basic Authentication (Legacy)
- Some endpoints support Basic Auth as alternative
- Username and password sent in `Authorization` header
- Format: `Authorization: Basic <base64(username:password)>`

## Error Responses

All endpoints return consistent error responses:

```json
{
  "error": "Error description",
  "details": "Additional error details (development only)"
}
```

## Rate Limiting

- Default rate limiting applied to all endpoints
- Stricter limits on authentication endpoints
- Rate limit headers included in responses

## Security Considerations

- All endpoints require HTTPS in production
- CSRF protection on state-changing operations
- Input validation and sanitization
- SQL injection prevention through parameterized queries
- XSS protection through output encoding

## Environment Configuration

Key environment variables for API configuration:

- `APP_APPLICATION__BASE_URL`: Application base URL
- `APP_DATABASE__*`: Database connection parameters
- `APP_EMAIL__*`: Email service configuration
- `APP_REDIS__URI`: Redis connection string
- `RUST_LOG`: Logging level configuration