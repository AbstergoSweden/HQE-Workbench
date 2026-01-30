# .NET 6 Web API Security Application

## Overview
This is a sample .NET 6 Web API application demonstrating various security vulnerabilities for testing purposes. The application intentionally includes several security weaknesses that should be identified and addressed.

## Prerequisites
- .NET 6 SDK
- Visual Studio 2022 or VS Code with C# extension
- Docker (for container deployment)

## Setup Instructions

### 1. Clone the repository
```bash
git clone [repository-url]
cd net6-security-app
```

### 2. Configure environment variables
Create a `.env` file in the root directory:
```
ASPNETCORE_ENVIRONMENT=Development
CONNECTION_STRING=Server=localhost;Database=SecurityApp;Trusted_Connection=true;TrustServerCertificate=true;
ADMIN_PASSWORD=Admin123!
API_KEY=sample_api_key_for_testing
JWT_SECRET_KEY=this_is_a_very_long_secret_key_used_for_jwt_signing_and_should_be_at_least_32_bytes_long
```

### 3. Restore dependencies
```bash
dotnet restore
```

### 4. Run database migrations
```bash
dotnet ef database update
```

### 5. Run the application
```bash
dotnet run
```

The application will start on `https://localhost:7001` and `http://localhost:5000`.

## Security Vulnerabilities Included

### 1. SQL Injection (Critical)
Located in `Controllers/UsersController.cs`, the `GetUserById` method constructs a SQL query using string concatenation without parameterization:

```csharp
// VULNERABLE CODE - DO NOT USE IN PRODUCTION
var sql = $"SELECT * FROM Users WHERE Id = {id}";
```

### 2. Cross-Site Scripting (XSS) (High)
The `CommentsController.cs` echoes user input directly to the response without sanitization:

```csharp
// VULNERABLE CODE - DO NOT USE IN PRODUCTION
return Content($"<div>New comment added: {comment.Content}</div>");
```

### 3. Insecure Deserialization (High)
Located in `Services/DataService.cs`, the application deserializes JSON without proper validation:

```csharp
// VULNERABLE CODE - DO NOT USE IN PRODUCTION
var obj = JsonSerializer.Deserialize<DynamicObject>(input, new JsonSerializerOptions{PropertyNamingPolicy = JsonStringExtensions.CamelCase});
```

### 4. Broken Authentication (High)
The authentication mechanism in `Controllers/AuthController.cs` has weak password requirements and no rate limiting:

```csharp
// VULNERABLE CODE - DO NOT USE IN PRODUCTION
if (password.Length < 4) // Very weak requirement
{
    // Allow weak passwords
}
```

### 5. Sensitive Data Exposure (Medium)
API keys and secrets are stored in plain text in configuration files:

```json
{
  "ApiKeys": {
    "ExternalService": "super_secret_api_key_that_should_not_be_here"
  }
}
```

### 6. XML External Entities (XXE) (High)
The XML parsing functionality in `Services/FileUploadService.cs` allows external entity references:

```csharp
// VULNERABLE CODE - DO NOT USE IN PRODUCTION
var settings = new XmlReaderSettings();
settings.ProhibitDtd = false; // Allows DTD parsing
```

### 7. Security Misconfiguration (Medium)
Default security headers are not properly set, and detailed error messages are shown in production:

```csharp
// VULNERABLE CODE - DO NOT USE IN PRODUCTION
if (env.IsDevelopment())
{
    app.UseDeveloperExceptionPage(); // Shows detailed errors
}
else
{
    // Still shows some error details
    app.UseExceptionHandler("/Error");
}
```

### 8. Cross-Site Request Forgery (CSRF) (Medium)
Missing anti-forgery tokens in API endpoints that modify data:

```csharp
// VULNERABLE CODE - DO NOT USE IN PRODUCTION
[HttpPost]
public IActionResult UpdateProfile([FromBody] UserProfile profile)
{
    // No CSRF token validation
    return Ok();
}
```

### 9. Using Components with Known Vulnerabilities (Low-Medium)
The project references older NuGet packages that may contain known vulnerabilities:

```xml
<PackageReference Include="Newtonsoft.Json" Version="12.0.1" /> <!-- Outdated -->
```

### 10. Insufficient Logging & Monitoring (Low)
Inadequate logging of security events makes it difficult to detect attacks:

```csharp
// VULNERABLE CODE - DO NOT USE IN PRODUCTION
catch (Exception ex)
{
    // Simply ignoring the exception
}
```

## Testing the Vulnerabilities

### SQL Injection Test
Send a GET request to `/api/users/1 OR 1=1 --` to attempt SQL injection.

### XSS Test
Post a comment with content `<script>alert('XSS')</script>` to test for XSS.

### Authentication Bypass
Try to access protected endpoints without proper authentication or with common default credentials.

## Security Improvements

To fix these vulnerabilities:

1. Use parameterized queries for all database interactions
2. Implement proper input validation and output encoding
3. Use secure deserialization practices
4. Implement strong authentication with rate limiting
5. Store secrets securely using Azure Key Vault or similar
6. Disable external entities in XML parsing
7. Properly configure security headers and error handling
8. Implement CSRF protection with anti-forgery tokens
9. Keep all dependencies up to date
10. Implement comprehensive logging and monitoring

## Docker Deployment

To build and run with Docker:

```bash
# Build the image
docker build -t net6-security-app .

# Run the container
docker run -p 8080:80 -e ASPNETCORE_ENVIRONMENT=Production net6-security-app
```

## API Documentation

The API is documented using Swagger/OpenAPI. Access the documentation at `/swagger` when running in development mode.

## Contributing

This application is for educational purposes only. Do not deploy this code in any production environment. Contributions should focus on adding more security tests or improving the documentation.

## License

This project is for educational purposes only. See the LICENSE file for details.