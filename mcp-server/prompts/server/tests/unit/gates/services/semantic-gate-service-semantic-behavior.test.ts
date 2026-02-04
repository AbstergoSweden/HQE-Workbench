import { describe, test, beforeEach, expect } from '@jest/globals';
import { SemanticGateService } from '../../../../src/gates/services/semantic-gate-service.js';
import { createMockLogger } from '../../../__mocks__/logger-mock.js';
import { createMockGateGuidanceRenderer } from '../../../__mocks__/gate-guidance-renderer-mock.js';
import { createMockGateValidator } from '../../../__mocks__/gate-validator-mock.js';
import type { ConvertedPrompt } from '../../../../src/types/index.js';
import type { GateContext } from '../../../../src/gates/core/gate-definitions.js';

describe('SemanticGateService - Semantic Validation Behavior', () => {
  let service: SemanticGateService;
  let mockLogger: any;
  let mockRenderer: any;
  let mockValidator: any;

  const mockPrompt: ConvertedPrompt = {
    id: 'test-prompt',
    category: 'test-category',
    userMessageTemplate: 'Test template',
    systemMessage: 'Test system message',
  };

  const mockContext: GateContext = {
    category: 'test-category',
    promptId: 'test-prompt-id',
  };

  beforeEach(() => {
    mockLogger = createMockLogger();
    mockRenderer = createMockGateGuidanceRenderer();
    mockValidator = createMockGateValidator();
  });

  test('should return validation results with semantic-skipped when semantic validation is enabled but not implemented (fail-open)', async () => {
    service = new SemanticGateService(mockLogger, mockRenderer, mockValidator);
    service.updateConfig({
      llmIntegration: { enabled: true },
      failClosedOnSemanticError: false  // Default fail-open behavior
    });

    const result = await service.enhancePrompt(mockPrompt, ['gate1', 'gate2'], mockContext);

    expect(result.validationResults).toBeDefined();
    expect(result.validationResults).toHaveLength(2);

    for (const validation of result.validationResults!) {
      expect(validation.gateId).toMatch(/^(gate1|gate2)$/);
      expect(validation.passed).toBe(true); // Should pass in fail-open mode
      expect(validation.score).toBe(0); // Zero confidence
      expect(validation.validatedBy).toBe('semantic-skipped');
      expect(validation.feedback).toContain('was skipped - LLM integration not implemented');
      expect(validation.feedback).toContain('fail-open mode');
    }
  });

  test('should return validation results with semantic-not-implemented when semantic validation is enabled but not implemented (fail-closed)', async () => {
    service = new SemanticGateService(mockLogger, mockRenderer, mockValidator);
    service.updateConfig({
      llmIntegration: { enabled: true },
      failClosedOnSemanticError: true  // Fail-closed behavior
    });

    const result = await service.enhancePrompt(mockPrompt, ['gate1', 'gate2'], mockContext);

    expect(result.validationResults).toBeDefined();
    expect(result.validationResults).toHaveLength(2);

    for (const validation of result.validationResults!) {
      expect(validation.gateId).toMatch(/^(gate1|gate2)$/);
      expect(validation.passed).toBe(false); // Should fail in fail-closed mode
      expect(validation.score).toBe(0); // Zero confidence
      expect(validation.validatedBy).toBe('semantic-not-implemented');
      expect(validation.feedback).toContain('failed - LLM integration not implemented');
      expect(validation.feedback).toContain('fail-closed mode');
    }
  });

  test('should return undefined validationResults when semantic validation is disabled', async () => {
    service = new SemanticGateService(mockLogger, mockRenderer, mockValidator);
    // Don't enable semantic validation - leave it as default (disabled)

    const result = await service.enhancePrompt(mockPrompt, ['gate1', 'gate2'], mockContext);

    // Should return compositional result only (no semantic validation)
    expect(result.validationResults).toBeUndefined();
  });

  test('should handle semantic validation errors according to failClosedOnSemanticError flag', async () => {
    // Create a service that will throw an error during semantic validation
    service = new SemanticGateService(mockLogger, mockRenderer, mockValidator);
    service.updateConfig({
      llmIntegration: { enabled: true },
      failClosedOnSemanticError: true  // This should trigger error handling
    });

    // Mock the performSemanticValidation to throw an error
    const originalMethod = (service as any).performSemanticValidation;
    (service as any).performSemanticValidation = async () => {
      throw new Error('Test semantic validation error');
    };

    const result = await service.enhancePrompt(mockPrompt, ['gate1'], mockContext);

    // With failClosedOnSemanticError=true, should return error validation results
    expect(result.validationResults).toBeDefined();
    expect(result.validationResults).toHaveLength(1);
    expect(result.validationResults![0].gateId).toBe('gate1');
    expect(result.validationResults![0].passed).toBe(false);
    expect(result.validationResults![0].validatedBy).toBe('semantic-error');
    expect(result.validationResults![0].feedback).toContain('Test semantic validation error');

    // Restore the original method
    (service as any).performSemanticValidation = originalMethod;
  });

  test('should degrade to compositional when semantic validation errors and failClosedOnSemanticError=false', async () => {
    service = new SemanticGateService(mockLogger, mockRenderer, mockValidator);
    service.updateConfig({
      llmIntegration: { enabled: true },
      failClosedOnSemanticError: false  // Default fail-open behavior
    });

    // Mock the performSemanticValidation to throw an error
    const originalMethod = (service as any).performSemanticValidation;
    (service as any).performSemanticValidation = async () => {
      throw new Error('Test semantic validation error');
    };

    const result = await service.enhancePrompt(mockPrompt, ['gate1'], mockContext);

    // With failClosedOnSemanticError=false, should degrade to compositional (no validation results)
    expect(result.validationResults).toBeUndefined();

    // Restore the original method
    (service as any).performSemanticValidation = originalMethod;
  });
});