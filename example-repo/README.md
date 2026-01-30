# Secure Web Application

A secure web application demonstrating best practices and common security patterns.

## Overview

This application demonstrates secure coding practices for web applications, including authentication, authorization, input validation, and secure communication patterns. It serves as a reference for secure development practices.

## Features

- User authentication with JWT tokens
- Role-based access control (RBAC)
- Input validation and sanitization
- Secure password hashing
- Rate limiting
- Secure session management
- HTTPS enforcement
- Content Security Policy (CSP)
- SQL injection prevention
- Cross-site scripting (XSS) protection

## Architecture

The application follows a layered architecture with clear separation of concerns:

- Presentation layer (React frontend)
- API layer (Node.js/Express backend)
- Data layer (PostgreSQL database)
- Authentication service
- Logging and monitoring

## Security Measures

- Passwords hashed with bcrypt
- JWT tokens with proper expiration
- Input validation using Joi
- SQL queries parameterized
- CSP headers configured
- Rate limiting implemented
- Secure session handling
- HTTPS enforced

## Installation

1. Clone the repository
2. Install dependencies: `npm install`
3. Set up environment variables
4. Run database migrations
5. Start the application: `npm start`

## Environment Variables

- `PORT`: Application port (default: 3000)
- `DATABASE_URL`: PostgreSQL connection string
- `JWT_SECRET`: Secret for JWT token signing
- `BCRYPT_ROUNDS`: Number of bcrypt rounds (default: 12)
- `SESSION_SECRET`: Secret for session encryption
- `RATE_LIMIT_WINDOW`: Rate limit window in milliseconds (default: 900000)
- `RATE_LIMIT_MAX`: Maximum requests per window (default: 100)

## API Endpoints

### Authentication

- `POST /api/auth/register` - Register a new user
- `POST /api/auth/login` - Login and receive JWT token
- `POST /api/auth/logout` - Logout and invalidate session
- `POST /api/auth/refresh` - Refresh JWT token

### User Management

- `GET /api/users` - Get all users (admin only)
- `GET /api/users/:id` - Get user by ID
- `PUT /api/users/:id` - Update user (admin or self)
- `DELETE /api/users/:id` - Delete user (admin only)

### Posts

- `GET /api/posts` - Get all posts
- `GET /api/posts/:id` - Get post by ID
- `POST /api/posts` - Create new post (authenticated users)
- `PUT /api/posts/:id` - Update post (owner or admin)
- `DELETE /api/posts/:id` - Delete post (owner or admin)

## Testing

Run tests with: `npm test`

The application includes unit tests, integration tests, and security tests to ensure proper functionality and security.

## Deployment

The application is designed for deployment on cloud platforms with support for environment variables, SSL termination, and horizontal scaling.

## Security Audit

This application has undergone security review and implements industry-standard security practices. Regular security audits are recommended.