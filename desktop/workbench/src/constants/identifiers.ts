/**
 * Global identifier constants and helpers
 */

export const API_KEY_PREFIX = 'api_key';

/**
 * Generates a consistent API key ID for a given profile name.
 * @param profileName The name of the profile
 * @returns The formatted API key ID (e.g., 'api_key:Venice')
 */
export const getApiKeyId = (profileName: string): string => {
    return `${API_KEY_PREFIX}:${profileName}`;
};

export const PROVIDER_IDS = {
    OPENAI: 'openai',
    ANTHROPIC: 'anthropic',
    VENICE: 'venice',
    OPENROUTER: 'openrouter',
    XAI_GROK: 'xai-grok',
    GOOGLE_GEMINI: 'google-gemini',
} as const;
