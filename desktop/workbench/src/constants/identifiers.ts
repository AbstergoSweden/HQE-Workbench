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
