import { describe, test, beforeEach, expect } from '@jest/globals';
import { SemanticGateService } from '../../../../src/gates/services/semantic-gate-service.js';
import { createMockLogger } from '../../../__mocks__/logger-mock.js';
import { createMockGateGuidanceRenderer } from '../../../__mocks__/gate-guidance-renderer-mock.js';
import { createMockGateValidator } from '../../../__mocks__/gate-validator-mock.js';
import type { GateServiceConfig } from '../../../../src/gates/services/gate-service-interface.js';

describe('SemanticGateService - Config Merging', () => {
  let service: SemanticGateService;
  let mockLogger: any;
  let mockRenderer: any;
  let mockValidator: any;

  beforeEach(() => {
    mockLogger = createMockLogger();
    mockRenderer = createMockGateGuidanceRenderer();
    mockValidator = createMockGateValidator();
  });

  test('should preserve nested llmIntegration defaults when partially updating config', () => {
    service = new SemanticGateService(mockLogger, mockRenderer, mockValidator);

    // Initially, llmIntegration should have default values
    const initialConfig = (service as any).config;
    expect(initialConfig.llmIntegration?.enabled).toBe(false);
    expect(initialConfig.llmIntegration?.model).toBe('default');
    expect(initialConfig.llmIntegration?.maxTokens).toBe(2048);
    expect(initialConfig.llmIntegration?.temperature).toBe(0.2);

    // Update only the enabled flag, leaving other fields untouched
    service.updateConfig({ llmIntegration: { enabled: true } });

    // Check that other fields remain unchanged
    const updatedConfig = (service as any).config;
    expect(updatedConfig.llmIntegration?.enabled).toBe(true);
    expect(updatedConfig.llmIntegration?.model).toBe('default'); // Should remain unchanged
    expect(updatedConfig.llmIntegration?.maxTokens).toBe(2048); // Should remain unchanged
    expect(updatedConfig.llmIntegration?.temperature).toBe(0.2); // Should remain unchanged
  });

  test('should allow overriding specific llmIntegration fields', () => {
    service = new SemanticGateService(mockLogger, mockRenderer, mockValidator);

    // Update specific fields
    service.updateConfig({
      llmIntegration: {
        model: 'gpt-4',
        maxTokens: 4096,
      },
    });

    const config = (service as any).config;
    expect(config.llmIntegration?.enabled).toBe(false); // Default should remain
    expect(config.llmIntegration?.model).toBe('gpt-4'); // Updated
    expect(config.llmIntegration?.maxTokens).toBe(4096); // Updated
    expect(config.llmIntegration?.temperature).toBe(0.2); // Default should remain
  });

  test('should handle constructor config merging correctly', () => {
    const customConfig: Partial<GateServiceConfig> = {
      llmIntegration: {
        enabled: true,
        model: 'custom-model',
      },
    };

    service = new SemanticGateService(mockLogger, mockRenderer, mockValidator, customConfig);

    const config = (service as any).config;
    expect(config.llmIntegration?.enabled).toBe(true);
    expect(config.llmIntegration?.model).toBe('custom-model');
    expect(config.llmIntegration?.maxTokens).toBe(2048); // Default should remain
    expect(config.llmIntegration?.temperature).toBe(0.2); // Default should remain
  });

  test('should include failClosedOnSemanticError in config', () => {
    service = new SemanticGateService(mockLogger, mockRenderer, mockValidator);

    const config = (service as any).config;
    expect(config.failClosedOnSemanticError).toBeDefined();
    expect(config.failClosedOnSemanticError).toBe(false); // Default should be false
  });

  test('should allow updating failClosedOnSemanticError', () => {
    service = new SemanticGateService(mockLogger, mockRenderer, mockValidator);

    service.updateConfig({ failClosedOnSemanticError: true });

    const config = (service as any).config;
    expect(config.failClosedOnSemanticError).toBe(true);
  });
});