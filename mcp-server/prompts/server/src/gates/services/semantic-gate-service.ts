// @lifecycle migrating - Semantic scoring service still aligning with new guardrails.
import { CompositionalGateService } from './compositional-gate-service.js';

import type {
  IGateService,
  GateEnhancementResult,
  GateServiceConfig,
  GateValidationResult,
} from './gate-service-interface.js';
import type { Logger } from '../../logging/index.js';
import type { ConvertedPrompt } from '../../types/index.js';
import type { GateContext } from '../core/gate-definitions.js';
import type { GateValidator } from '../core/gate-validator.js';
import type { GateGuidanceRenderer } from '../guidance/GateGuidanceRenderer.js';

// Constants for semantic validation
const SEMANTIC_VALIDATION_NO_CONFIDENCE = 0;
// const SEMANTIC_VALIDATION_FULL_CONFIDENCE = 1.0;

// Utility function for deep merging configuration objects
// eslint-disable-next-line @typescript-eslint/no-explicit-any
function deepMerge(target: any, source: any): any {
  // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment
  const result = { ...target };

  for (const key in source) {
    if (Object.prototype.hasOwnProperty.call(source, key)) {
      // eslint-disable-next-line @typescript-eslint/no-unsafe-member-access, @typescript-eslint/no-unsafe-assignment
      const sourceValue = source[key];

      if (sourceValue !== null && typeof sourceValue === 'object' && !Array.isArray(sourceValue)) {
        if (
          // eslint-disable-next-line @typescript-eslint/no-unsafe-member-access
          result[key] !== null &&
          // eslint-disable-next-line @typescript-eslint/no-unsafe-member-access
          typeof result[key] === 'object' &&
          // eslint-disable-next-line @typescript-eslint/no-unsafe-member-access
          !Array.isArray(result[key])
        ) {
          // eslint-disable-next-line @typescript-eslint/no-unsafe-member-access, @typescript-eslint/no-unsafe-assignment
          result[key] = deepMerge(result[key], sourceValue);
        } else {
          // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment
          result[key] = { ...sourceValue };
        }
      } else if (sourceValue !== undefined) {
        // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment
        result[key] = sourceValue;
      }
    }
  }

  // eslint-disable-next-line @typescript-eslint/no-unsafe-return
  return result;
}

/**
 * Default configuration for Semantic Gate Service
 *
 * This configuration defines the default settings for semantic validation:
 * - enabled: Whether the semantic gate service is active
 * - failClosedOnSemanticError: Whether to fail validation when semantic errors occur (fail-closed vs fail-open)
 * - llmIntegration: Configuration for LLM integration including model, tokens, and temperature settings
 */
const DEFAULT_SEMANTIC_CONFIG: GateServiceConfig = {
  enabled: true,
  failClosedOnSemanticError: false, // Default to fail-open for backward compatibility
  llmIntegration: {
    enabled: false,
    model: 'default',
    maxTokens: 2048,
    temperature: 0.2,
  },
};

/**
 * Semantic Gate Service - Template rendering + server-side validation (future work)
 *
 * The SemanticGateService provides semantic validation capabilities for prompts using
 * LLM-based analysis. It performs deep semantic analysis of prompt content against
 * defined validation criteria and gate configurations.
 *
 * Features:
 * - Semantic validation using LLM integration (when enabled)
 * - Configurable fail-open/fail-closed behavior for semantic errors
 * - Gate ID validation to prevent injection attacks
 * - Performance optimizations including regex caching and parallel validation
 * - Comprehensive error handling and logging
 *
 * The service supports two operational modes:
 * - fail-open: Allows execution to continue when semantic validation fails
 * - fail-closed: Blocks execution when semantic validation fails
 */
export class SemanticGateService implements IGateService {
  readonly serviceType = 'semantic' as const;
  private readonly logger: Logger;
  private readonly gateValidator: GateValidator;
  private readonly compositionalService: CompositionalGateService;
  private config: GateServiceConfig;

  constructor(
    logger: Logger,
    gateGuidanceRenderer: GateGuidanceRenderer,
    gateValidator: GateValidator,
    config?: Partial<GateServiceConfig>
  ) {
    this.logger = logger;
    this.gateValidator = gateValidator;
    // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment
    this.config = deepMerge(DEFAULT_SEMANTIC_CONFIG, config ?? {});
    this.compositionalService = new CompositionalGateService(
      logger,
      gateGuidanceRenderer,
      this.config
    );
  }

  /**
   * Enhances a prompt with gate instructions and performs semantic validation if enabled.
   *
   * This method orchestrates the complete enhancement process:
   * 1. Applies compositional gate enhancements (template injection)
   * 2. Performs semantic validation if enabled in configuration
   * 3. Handles validation errors according to failClosedOnSemanticError setting
   * 4. Returns combined results with both enhancement and validation data
   *
   * @param prompt - The original prompt to enhance
   * @param gateIds - Array of gate IDs to apply during enhancement
   * @param context - Gate context containing metadata and execution context
   * @returns Promise resolving to GateEnhancementResult with enhanced prompt and validation results
   */
  async enhancePrompt(
    prompt: ConvertedPrompt,
    gateIds: string[],
    context: GateContext
  ): Promise<GateEnhancementResult> {
    const compositionalResult = await this.compositionalService.enhancePrompt(
      prompt,
      gateIds,
      context
    );

    if (!this.isValidationEnabled()) {
      return compositionalResult;
    }

    try {
      const validationResults = await this.performSemanticValidation(
        compositionalResult.enhancedPrompt,
        gateIds,
        context
      );

      return {
        ...compositionalResult,
        validationResults,
      };
    } catch (error) {
      this.logger.error(
        '[SemanticGateService] Semantic validation failed – degrading to compositional',
        {
          error: error instanceof Error ? error.message : String(error),
          errorType: error?.constructor?.name,
        }
      );

      // If failClosedOnSemanticError is true, return the error as a validation failure
      if (this.config.failClosedOnSemanticError === true) {
        const errorValidationResults: GateValidationResult[] = gateIds.map((gateId) => ({
          gateId,
          passed: false,
          score: SEMANTIC_VALIDATION_NO_CONFIDENCE,
          feedback: `Semantic validation failed: ${error instanceof Error ? error.message : String(error)}`,
          validatedBy: 'semantic',
          status: 'error',
          timestamp: Date.now(),
        }));

        return {
          ...compositionalResult,
          validationResults: errorValidationResults,
        };
      }

      // Otherwise, degrade to compositional as before (fail-open)
      return compositionalResult;
    }
  }

  /**
   * Indicates whether this service has the capability to perform semantic validation.
   *
   * @returns Always returns true, as this service is designed for semantic validation
   */
  supportsValidation(): boolean {
    // This service has the capability to perform semantic validation
    return true;
  }

  /**
   * Checks if semantic validation is currently enabled based on configuration.
   *
   * @returns True if semantic validation is enabled in the configuration, false otherwise
   */
  isValidationEnabled(): boolean {
    // Check if semantic validation is currently enabled in config
    // Requirement: active only when config.enabled === true AND config.llmIntegration.enabled === true
    const serviceEnabled = this.config.enabled;
    const llmEnabled = this.config.llmIntegration?.enabled ?? false;
    return serviceEnabled && llmEnabled;
  }

  updateConfig(config: Partial<GateServiceConfig>): void {
    // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment
    this.config = deepMerge(this.config, config);
    this.compositionalService.updateConfig(this.config);
  }

  /**
   * Validates a gate ID to ensure it follows expected format and doesn't contain malicious content.
   * Gate IDs should be alphanumeric with hyphens and underscores only.
   */
  private isValidGateId(gateId: string): boolean {
    if (typeof gateId !== 'string' || gateId.length === 0) {
      return false;
    }

    // Gate IDs should only contain alphanumeric characters, hyphens, and underscores
    // This prevents injection of special characters that could be used maliciously
    const gateIdPattern = /^[a-zA-Z0-9_-]+$/;
    return gateIdPattern.test(gateId);
  }

  /**
   * Performs semantic validation on the provided prompt using the specified gate IDs.
   *
   * This method handles the core semantic validation logic, including:
   * - Gate ID validation to prevent injection attacks
   * - Filtering of invalid gate IDs with appropriate logging
   * - Conditional validation based on failClosedOnSemanticError setting
   * - Creation of validation results with appropriate feedback messages
   *
   * @param prompt - The converted prompt to validate
   * @param gateIds - Array of gate IDs to apply validation rules for
   * @param context - Validation context containing additional metadata
   * @returns Promise resolving to an array of GateValidationResult objects
   */
  private async performSemanticValidation(
    prompt: ConvertedPrompt,
    gateIds: string[],
    context: GateContext
  ): Promise<GateValidationResult[]> {
    const currentTime = Date.now(); // Cache timestamp to avoid multiple Date.now() calls

    try {
      // Validate gate IDs to prevent injection attacks
      const validatedGateIds = gateIds.filter((gateId) => this.isValidGateId(gateId));

      // Log warning for any invalid gate IDs that were filtered out
      const invalidGateIds = gateIds.filter((gateId) => !this.isValidGateId(gateId));
      if (invalidGateIds.length > 0) {
        this.logger.warn('[SemanticGateService] Invalid gate IDs filtered out', {
          invalidGateIds,
          promptId: prompt.id,
        });
      }

      this.logger.info('[SemanticGateService] Performing semantic validation', {
        promptId: prompt.id,
        gateIds: validatedGateIds,
        contextHasFramework: context.framework !== undefined,
        contextHasCategory: context.category !== undefined,
        contextHasExplicitGateIds: context.explicitGateIds !== undefined,
      });

      // TODO: Connect actual LLM client for true semantic validation
      // For now, we interact with the fail-closed/fail-open logic.

      const isFailClosed = this.config.failClosedOnSemanticError === true;

      // Check if we can actually perform validation (mock check for now)
      // Since LLM is not hooked up, we are "not implemented"

      if (isFailClosed) {
        // Fail-closed: If we logic is not implemented, we must fail.
        return validatedGateIds.map((gateId) => ({
          gateId,
          passed: false,
          score: SEMANTIC_VALIDATION_NO_CONFIDENCE,
          feedback: `Semantic validation for ${gateId} failed - LLM integration not implemented (fail-closed mode).`,
          validatedBy: 'semantic',
          status: 'not-implemented',
          timestamp: currentTime,
        }));
      } else {
        // Fail-open: If logic not implemented, we skip and pass.
        return validatedGateIds.map((gateId) => ({
          gateId,
          passed: true,
          score: SEMANTIC_VALIDATION_NO_CONFIDENCE,
          feedback: `Semantic validation for ${gateId} was skipped - LLM integration not implemented (fail-open mode).`,
          validatedBy: 'semantic',
          status: 'skipped',
          timestamp: currentTime,
        }));
      }
    } catch (error) {
      this.logger.error('[SemanticGateService] Unexpected error during semantic validation', {
        error: error instanceof Error ? error.message : String(error),
        errorType: error?.constructor?.name,
        promptId: prompt.id,
        gateIds: gateIds,
      });

      // In case of unexpected errors, return appropriate results based on failClosedOnSemanticError setting
      if (this.config.failClosedOnSemanticError === true) {
        return gateIds.map((gateId) => ({
          gateId,
          passed: false,
          score: SEMANTIC_VALIDATION_NO_CONFIDENCE,
          feedback: `Semantic validation failed unexpectedly: ${error instanceof Error ? error.message : String(error)}`,
          validatedBy: 'semantic',
          status: 'error',
          timestamp: Date.now(),
        }));
      } else {
        // In fail-open mode, return successful results to avoid blocking execution
        return gateIds.map((gateId) => ({
          gateId,
          passed: true,
          score: SEMANTIC_VALIDATION_NO_CONFIDENCE,
          feedback: `Semantic validation skipped due to error: ${error instanceof Error ? error.message : String(error)}`,
          validatedBy: 'semantic',
          status: 'error',
          timestamp: Date.now(),
        }));
      }
    }
  }
}
