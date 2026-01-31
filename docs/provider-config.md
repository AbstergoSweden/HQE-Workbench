# Provider Configuration

HQE Workbench supports any OpenAI-compatible API provider.

## Supported Providers

### Venice.ai
```
Name: venice
Base URL: https://api.venice.ai/api/v1
Models: Discover via Settings; filters to text/code models only
```

### OpenAI
```
Name: openai
Base URL: https://api.openai.com/v1
Models: gpt-4o, gpt-4o-mini, gpt-4-turbo, gpt-3.5-turbo
```

### Azure OpenAI
```
Name: azure-openai
Base URL: https://my-resource.openai.azure.com/openai/deployments/my-deployment
Models: Use your deployment name from the Azure OpenAI portal
Headers: api-key: (Azure OpenAI key from the portal)
```

### LocalAI / LM Studio
```
Name: local
Base URL: http://localhost:1234/v1
Models: Any local model
```

### Ollama
```
Name: ollama
Base URL: http://localhost:11434/v1
Models: llama2, codellama, etc.
```

## Configuration Storage

### API Keys
- Stored in macOS Keychain
- Never written to disk in plain text
- Keychain item ID format: `api_key:{profile_name}`

### Profile Settings
- Stored in: `~/.local/share/hqe-workbench/profiles.json`
- Contains: name, base_url, model, headers (without keys)

## Testing Connection

The app can test connectivity before saving:
1. Sends minimal chat request: "Hi"
2. Expects any valid response
3. Reports success/failure

## Troubleshooting

### Connection Failed
- Verify URL is correct
- Check API key is valid
- Ensure firewall allows connection
- For local: verify service is running

### Rate Limited
- Implement exponential backoff
- Check provider limits
- Consider batching requests

### Timeout
- Default: 60 seconds
- Can be configured per profile
- Check network connectivity
