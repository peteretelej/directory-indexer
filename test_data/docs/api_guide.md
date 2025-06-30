# API Documentation Guide

This document explains how to use the REST API endpoints for user authentication and data management.

## Authentication

All API requests require authentication using Bearer tokens in the Authorization header.

```bash
curl -H "Authorization: Bearer your-token-here" https://api.example.com/users
```

## Endpoints

### User Management

- `GET /users` - List all users
- `POST /users` - Create new user
- `PUT /users/{id}` - Update user
- `DELETE /users/{id}` - Delete user

### Error Handling

The API returns standard HTTP status codes:
- 200: Success
- 400: Bad Request
- 401: Unauthorized  
- 404: Not Found
- 500: Internal Server Error

## Rate Limiting

API requests are limited to 1000 per hour per authentication token.