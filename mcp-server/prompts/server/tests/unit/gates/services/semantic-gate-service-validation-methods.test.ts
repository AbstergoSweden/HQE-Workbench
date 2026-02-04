import { describe, test, beforeEach, expect } from '@jest/globals';
import { SemanticGateService } from '../../../../src/gates/services/semantic-gate-service.js';
import { createMockLogger } from '../../../__mocks__/logger-mock.js';
import { createMockGateGuidanceRenderer } from '../../../__mocks__/gate-guidance-renderer-mock.js';
import { createMockGateValidator } from '../../../__mocks__/gate-validator-mock.js';

describe('SemanticGateService - Validation Capability vs Enabled', () => {
  let service: SemanticGateService;
  let mockLogger: any;
  let mockRenderer: any;
  let mockValidator: any;

  beforeEach(() => {
    mockLogger = createMockLogger();
    mockRenderer = createMockGateGuidanceRenderer();
    mockValidator = createMockGateValidator();
  });

  test('should always return true for supportsValidation (capability)', () => {
    service = new SemanticGateService(mockLogger, mockRenderer, mockValidator);

    // Even when semantic validation is disabled, the service should support it (has capability)
    expect(service.supportsValidation()).toBe(true);

    // Even when semantic validation is enabled, the service should still support it
    service.updateConfig({ llmIntegration: { enabled: true } });
    expect(service.supportsValidation()).toBe(true);
  });

  test('should return correct state for isValidationEnabled', () => {
    service = new SemanticGateService(mockLogger, mockRenderer, mockValidator);

    // By default, semantic validation should be disabled
    expect(service.isValidationEnabled()).toBe(false);

    // When enabled, should return true
    service.updateConfig({ llmIntegration: { enabled: true } });
    expect(service.isValidationEnabled()).toBe(true);

    // When disabled again, should return false
    service.updateConfig({ llmIntegration: { enabled: false } });
    expect(service.isValidationEnabled()).toBe(false);
  });

  test('should distinguish between capability and enabled state', () => {
    service = new SemanticGateService(mockLogger, mockRenderer, mockValidator);

    // Capability should always be true regardless of enabled state
    expect(service.supportsValidation()).toBe(true);
    expect(service.isValidationEnabled()).toBe(false);

    service.updateConfig({ llmIntegration: { enabled: true } });
    expect(service.supportsValidation()).toBe(true);
    expect(service.isValidationEnabled()).toBe(true);
  });
});