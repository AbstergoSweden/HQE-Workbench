# API Documentation

## Overview

This document describes the API endpoints for the Secure Web Application. All API requests should be made to the base URL: `https://api.securewebapp.com/v1`

## Authentication

All API requests require authentication using a Bearer token. Include the token in the Authorization header:

```
Authorization: Bearer <your-api-token>
```

To obtain an API token, first authenticate with the `/auth/login` endpoint.

## Rate Limiting

The API implements rate limiting to prevent abuse:

- 100 requests per 15-minute window per IP address
- 1000 requests per hour per authenticated user
- Exceeding limits results in a 429 (Too Many Requests) response

## Error Handling

All API responses follow this structure:

```json
{
  "success": true,
  "data": {},
  "error": null
}
```

For error responses:

```json
{
  "success": false,
  "data": null,
  "error": {
    "code": "ERROR_CODE",
    "message": "Human-readable error message",
    "details": {}
  }
}
```

### Common Error Codes

- `INVALID_INPUT`: Request data failed validation
- `UNAUTHORIZED`: Authentication required or failed
- `FORBIDDEN`: Insufficient permissions
- `NOT_FOUND`: Requested resource does not exist
- `RATE_LIMIT_EXCEEDED`: Too many requests
- `INTERNAL_ERROR`: Server-side error

## Endpoints

### Authentication

#### POST /auth/register

Register a new user account.

**Request Body:**
```json
{
  "username": "johndoe",
  "email": "john@example.com",
  "password": "securePassword123",
  "confirmPassword": "securePassword123"
}
```

**Validation:**
- Username: 3-30 alphanumeric characters
- Email: Valid email format
- Password: At least 8 characters with uppercase, lowercase, number, and special character

**Response:**
```json
{
  "success": true,
  "data": {
    "user": {
      "id": "uuid",
      "username": "johndoe",
      "email": "john@example.com",
      "role": "user",
      "createdAt": "2023-01-01T00:00:00.000Z"
    },
    "token": "jwt-token"
  },
  "error": null
}
```

#### POST /auth/login

Authenticate and receive an access token.

**Request Body:**
```json
{
  "email": "john@example.com",
  "password": "securePassword123"
}
```

**Response:**
```json
{
  "success": true,
  "data": {
    "user": {
      "id": "uuid",
      "username": "johndoe",
      "email": "john@example.com",
      "role": "user",
      "lastLogin": "2023-01-01T00:00:00.000Z"
    },
    "token": "jwt-token",
    "refreshToken": "refresh-token"
  },
  "error": null
}
```

#### POST /auth/refresh

Refresh an access token using a refresh token.

**Request Headers:**
```
Authorization: Bearer <refresh-token>
```

**Response:**
```json
{
  "success": true,
  "data": {
    "token": "new-jwt-token"
  },
  "error": null
}
```

#### POST /auth/logout

Invalidate the current session.

**Request Headers:**
```
Authorization: Bearer <access-token>
```

**Response:**
```json
{
  "success": true,
  "data": {
    "message": "Successfully logged out"
  },
  "error": null
}
```

#### POST /auth/forgot-password

Initiate password reset process.

**Request Body:**
```json
{
  "email": "john@example.com"
}
```

**Response:**
```json
{
  "success": true,
  "data": {
    "message": "Password reset email sent if account exists"
  },
  "error": null
}
```

#### POST /auth/reset-password

Complete password reset process.

**Request Body:**
```json
{
  "token": "reset-token-from-email",
  "password": "newSecurePassword123",
  "confirmPassword": "newSecurePassword123"
}
```

**Response:**
```json
{
  "success": true,
  "data": {
    "message": "Password reset successfully"
  },
  "error": null
}
```

### Users

#### GET /users/me

Get current user's profile.

**Request Headers:**
```
Authorization: Bearer <access-token>
```

**Response:**
```json
{
  "success": true,
  "data": {
    "user": {
      "id": "uuid",
      "username": "johndoe",
      "email": "john@example.com",
      "role": "user",
      "isActive": true,
      "lastLogin": "2023-01-01T00:00:00.000Z",
      "profile": {
        "firstName": "John",
        "lastName": "Doe",
        "bio": "Software developer",
        "avatar": "https://example.com/avatar.jpg"
      }
    }
  },
  "error": null
}
```

#### PUT /users/me

Update current user's profile.

**Request Headers:**
```
Authorization: Bearer <access-token>
```

**Request Body:**
```json
{
  "firstName": "John",
  "lastName": "Doe",
  "bio": "Senior software developer",
  "timezone": "America/New_York",
  "language": "en"
}
```

**Response:**
```json
{
  "success": true,
  "data": {
    "user": {
      "id": "uuid",
      "username": "johndoe",
      "email": "john@example.com",
      "role": "user",
      "profile": {
        "firstName": "John",
        "lastName": "Doe",
        "bio": "Senior software developer",
        "timezone": "America/New_York",
        "language": "en"
      }
    }
  },
  "error": null
}
```

#### GET /users/:id

Get a specific user's public profile.

**Request Headers:**
```
Authorization: Bearer <access-token>
```

**Parameters:**
- `id`: User ID

**Response:**
```json
{
  "success": true,
  "data": {
    "user": {
      "id": "uuid",
      "username": "janedoe",
      "role": "user",
      "profile": {
        "firstName": "Jane",
        "lastName": "Doe",
        "bio": "Designer",
        "avatar": "https://example.com/avatar.jpg"
      }
    }
  },
  "error": null
}
```

### Posts

#### GET /posts

Get a list of published posts.

**Query Parameters:**
- `page` (optional): Page number (default: 1)
- `limit` (optional): Items per page (default: 10, max: 50)
- `category` (optional): Filter by category
- `author` (optional): Filter by author ID
- `sort` (optional): Sort by `newest`, `oldest`, `popular` (default: newest)

**Response:**
```json
{
  "success": true,
  "data": {
    "posts": [
      {
        "id": "uuid",
        "title": "Getting Started with Security",
        "excerpt": "Learn the basics of application security...",
        "author": {
          "id": "uuid",
          "username": "johndoe",
          "avatar": "https://example.com/avatar.jpg"
        },
        "publishedAt": "2023-01-01T00:00:00.000Z",
        "category": "security",
        "tags": ["security", "web", "development"],
        "viewCount": 125,
        "likeCount": 12,
        "commentCount": 3
      }
    ],
    "pagination": {
      "currentPage": 1,
      "totalPages": 5,
      "totalItems": 50,
      "itemsPerPage": 10
    }
  },
  "error": null
}
```

#### GET /posts/:id

Get a specific post by ID.

**Request Headers:**
```
Authorization: Bearer <access-token> (optional)
```

**Parameters:**
- `id`: Post ID

**Response:**
```json
{
  "success": true,
  "data": {
    "post": {
      "id": "uuid",
      "title": "Getting Started with Security",
      "content": "# Security Basics\n\nSecurity is important...",
      "author": {
        "id": "uuid",
        "username": "johndoe",
        "avatar": "https://example.com/avatar.jpg"
      },
      "publishedAt": "2023-01-01T00:00:00.000Z",
      "category": "security",
      "tags": ["security", "web", "development"],
      "viewCount": 125,
      "likeCount": 12,
      "commentCount": 3,
      "readingTime": 5
    }
  },
  "error": null
}
```

#### POST /posts

Create a new post.

**Request Headers:**
```
Authorization: Bearer <access-token>
Content-Type: application/json
```

**Request Body:**
```json
{
  "title": "My New Post",
  "content": "# Introduction\n\nThis is my new post...",
  "excerpt": "Brief description of the post",
  "category": "development",
  "tags": ["javascript", "security"],
  "published": false,
  "featured": false
}
```

**Response:**
```json
{
  "success": true,
  "data": {
    "post": {
      "id": "uuid",
      "title": "My New Post",
      "content": "# Introduction\n\nThis is my new post...",
      "excerpt": "Brief description of the post",
      "authorId": "current-user-id",
      "published": false,
      "featured": false,
      "category": "development",
      "tags": ["javascript", "security"],
      "createdAt": "2023-01-01T00:00:00.000Z",
      "updatedAt": "2023-01-01T00:00:00.000Z"
    }
  },
  "error": null
}
```

#### PUT /posts/:id

Update an existing post.

**Request Headers:**
```
Authorization: Bearer <access-token>
Content-Type: application/json
```

**Parameters:**
- `id`: Post ID

**Request Body:**
```json
{
  "title": "Updated Post Title",
  "content": "# Updated Content\n\nThis is the updated post...",
  "excerpt": "Updated brief description",
  "category": "security",
  "tags": ["javascript", "security", "best-practices"],
  "published": true
}
```

**Response:**
```json
{
  "success": true,
  "data": {
    "post": {
      "id": "uuid",
      "title": "Updated Post Title",
      "content": "# Updated Content\n\nThis is the updated post...",
      "excerpt": "Updated brief description",
      "authorId": "current-user-id",
      "published": true,
      "publishedAt": "2023-01-01T00:00:00.000Z",
      "featured": false,
      "category": "security",
      "tags": ["javascript", "security", "best-practices"],
      "createdAt": "2023-01-01T00:00:00.000Z",
      "updatedAt": "2023-01-01T00:00:00.000Z"
    }
  },
  "error": null
}
```

#### DELETE /posts/:id

Delete a post.

**Request Headers:**
```
Authorization: Bearer <access-token>
```

**Parameters:**
- `id`: Post ID

**Response:**
```json
{
  "success": true,
  "data": {
    "message": "Post deleted successfully"
  },
  "error": null
}
```

### Comments

#### GET /posts/:postId/comments

Get comments for a specific post.

**Query Parameters:**
- `page` (optional): Page number (default: 1)
- `limit` (optional): Items per page (default: 10, max: 50)
- `sort` (optional): Sort by `newest`, `oldest` (default: newest)

**Response:**
```json
{
  "success": true,
  "data": {
    "comments": [
      {
        "id": "uuid",
        "content": "Great post!",
        "author": {
          "id": "uuid",
          "username": "commenter",
          "avatar": "https://example.com/avatar.jpg"
        },
        "createdAt": "2023-01-01T00:00:00.000Z",
        "parentId": null,
        "likeCount": 2
      }
    ],
    "pagination": {
      "currentPage": 1,
      "totalPages": 1,
      "totalItems": 1,
      "itemsPerPage": 10
    }
  },
  "error": null
}
```

#### POST /posts/:postId/comments

Add a comment to a post.

**Request Headers:**
```
Authorization: Bearer <access-token>
Content-Type: application/json
```

**Request Body:**
```json
{
  "content": "This is a great article, thanks for sharing!",
  "parentId": null
}
```

**Response:**
```json
{
  "success": true,
  "data": {
    "comment": {
      "id": "uuid",
      "content": "This is a great article, thanks for sharing!",
      "author": {
        "id": "current-user-id",
        "username": "currentuser",
        "avatar": "https://example.com/avatar.jpg"
      },
      "postId": "post-id",
      "parentId": null,
      "createdAt": "2023-01-01T00:00:00.000Z"
    }
  },
  "error": null
}
```

## Security Headers

All API responses include security headers:

- `X-Content-Type-Options: nosniff`
- `X-Frame-Options: DENY`
- `X-XSS-Protection: 1; mode=block`
- `Strict-Transport-Security: max-age=31536000; includeSubDomains`
- `Content-Security-Policy: default-src 'self'; ...`
- `Referrer-Policy: no-referrer`

## Versioning

The API uses URI versioning. All endpoints are prefixed with `/v1` indicating the first version of the API.

Future breaking changes will introduce new versions (e.g., `/v2`) while maintaining older versions for a transition period.

## SDKs and Libraries

Official SDKs are available for:

- JavaScript/Node.js: `@securewebapp/api-client`
- Python: `securewebapp-api`
- Java: `com.securewebapp:api-client`

## Support

For API support, contact [api-support@securewebapp.com](mailto:api-support@securewebapp.com) or visit our [developer portal](https://developers.securewebapp.com).