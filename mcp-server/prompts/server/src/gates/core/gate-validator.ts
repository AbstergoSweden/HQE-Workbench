// @lifecycle canonical - Validates gate definitions before execution.
/**
 * Core Gate Validator
 *
 * Provides the validation infrastructure for gate-based quality control.
 *
 * DESIGN DECISION: String-based validation removed
 * ------------------------------------------------
 * Naive checks like length validation, substring matching, and regex patterns
 * have been intentionally removed. These don't provide meaningful signal for
 * LLM-generated content - an output can pass all string checks while being
 * semantically incorrect, or fail them while being excellent.
 *
 * The only validation that can meaningfully assess LLM output is LLM-based
 * evaluation (llm_self_check). The infrastructure remains in place for when
 * LLM integration is implemented.
 *
 * What's preserved:
 * - Validation framework and gate loading
 * - Statistics tracking and retry logic
 * - LLM self-check stub (TODO for implementation)
 * - Retry hints generation
 */

import { Logger } from '../../logging/index.js';

import type { GateDefinitionProvider } from './gate-loader.js';
import type { ValidationResult } from '../../execution/types.js';
import type { LLMIntegrationConfig } from '../../types.js';
import type {
  LightweightGateDefinition,
  ValidationCheck,
  ValidationContext,
  GatePassCriteria,
} from '../types.js';

/**
 * Gate validation statistics
 */
export interface GateValidationStatistics {
  totalValidations: number;
  successfulValidations: number;
  failedValidations: number;
  averageValidationTime: number;
  retryRequests: number;
}

/**
 * Core gate validator with pass/fail logic
 *
 * The GateValidator provides the core validation infrastructure for gate-based quality control.
 * It handles validation of content against various criteria types including content checks,
 * pattern matching, and LLM-based self-checks.
 *
 * Key features:
 * - Validation framework and gate loading
 * - Statistics tracking and retry logic
 * - LLM self-check stub (for future implementation)
 * - Retry hints generation
 * - Comprehensive error handling and logging
 * - Regex pattern caching for performance optimization
 */
export class GateValidator {
  private logger: Logger;
  private gateLoader: GateDefinitionProvider;
  private llmConfig: LLMIntegrationConfig | undefined;
  private readonly regexCache: Map<string, RegExp> = new Map();
  private validationStats: GateValidationStatistics = {
    totalValidations: 0,
    successfulValidations: 0,
    failedValidations: 0,
    averageValidationTime: 0,
    retryRequests: 0,
  };
  private validationTimes: number[] = [];

  constructor(
    logger: Logger,
    gateLoader: GateDefinitionProvider,
    llmConfig?: LLMIntegrationConfig
  ) {
    this.logger = logger;
    this.gateLoader = gateLoader;
    this.llmConfig = llmConfig;
  }

  /**
   * Validate content against a gate
   */
  async validateGate(gateId: string, context: ValidationContext): Promise<ValidationResult | null> {
    const startTime = Date.now();

    try {
      const gate = await this.gateLoader.loadGate(gateId);
      if (gate === null) {
        this.logger.warn(`Gate not found for validation: ${gateId}`);
        return null;
      }

      if (gate.type !== 'validation') {
        this.logger.debug(`Gate ${gateId} is guidance-only, skipping validation`);
        return {
          valid: true,
          passed: true,
          gateId,
          checks: [],
          retryHints: [],
          metadata: {
            validationTime: Date.now() - startTime,
            checksPerformed: 0,
            llmValidationUsed: false,
          },
        };
      }

      this.logger.debug(`Validating content against gate: ${gateId}`);

      // Run validation checks
      const checks: ValidationCheck[] = (context.metadata?.['checks'] as ValidationCheck[]) || [];
      let llmValidationUsed = false;

      if (gate.pass_criteria !== undefined) {
        for (const criteria of gate.pass_criteria) {
          const check = await this.runValidationCheck(criteria, context);
          checks.push(check);

          if (criteria.type === 'llm_self_check') {
            llmValidationUsed = true;
          }
        }
      }

      // Determine overall pass/fail
      const passed = checks.length === 0 || checks.every((check) => check.passed);

      // Generate retry hints for failures
      const retryHints = passed ? [] : this.generateRetryHints(gate, checks);

      const result: ValidationResult = {
        valid: passed,
        passed,
        gateId,
        checks,
        retryHints,
        metadata: {
          validationTime: Date.now() - startTime,
          checksPerformed: checks.length,
          llmValidationUsed,
        },
      };

      this.logger.debug(
        `Gate validation complete: ${gateId} - ${passed ? 'PASSED' : 'FAILED'} (${checks.length} checks)`
      );

      return result;
    } catch (error) {
      this.logger.error(`Gate validation failed for ${gateId}:`, error);
      return {
        valid: false,
        passed: false,
        gateId,
        checks: [
          {
            type: 'system_error',
            passed: false,
            message: `Validation error: ${error instanceof Error ? error.message : String(error)}`,
          },
        ],
        retryHints: [`Gate validation encountered an error. Please try again.`],
        metadata: {
          validationTime: Date.now() - startTime,
          checksPerformed: 0,
          llmValidationUsed: false,
        },
      };
    }
  }

  /**
   * Validate content against multiple gates
   */
  async validateGates(gateIds: string[], context: ValidationContext): Promise<ValidationResult[]> {
    const startTime = Date.now();

    // Run validations in parallel for better performance
    const validationPromises = gateIds.map(gateId => this.validateGate(gateId, context));
    const allResults = await Promise.all(validationPromises);

    // Filter out null results and update statistics
    const results: ValidationResult[] = [];
    for (const result of allResults) {
      if (result !== null) {
        results.push(result);

        // Update statistics based on result
        if (result.passed === true) {
          this.validationStats.successfulValidations++;
        } else {
          this.validationStats.failedValidations++;
        }
      }
    }

    // Update overall statistics
    const executionTime = Date.now() - startTime;
    this.validationTimes.push(executionTime);
    this.validationStats.totalValidations++;
    this.updateAverageValidationTime();

    return results;
  }

  /**
   * Run a single validation check
   *
   * NOTE: String-based checks (content_check, pattern_check) are retained as a
   * baseline for validation when LLM self-check is not configured. When LLM
   * integration is enabled, llm_self_check provides the strongest signal.
   */
  private async runValidationCheck(
    criteria: GatePassCriteria,
    context: ValidationContext
  ): Promise<ValidationCheck> {
    try {
      switch (criteria.type) {
        case 'llm_self_check':
          return await this.runLLMSelfCheck(criteria, context);

        case 'content_check':
          return this.runContentCheck(criteria, context);

        case 'pattern_check':
          return this.runPatternCheck(criteria, context);

        case 'methodology_compliance':
          return {
            type: criteria.type,
            passed: true,
            score: 1.0,
            message: 'Methodology compliance check deferred to framework system',
            details: {
              skipped: true,
              reason: 'Methodology compliance validation not implemented in gate validator',
            },
          };

        default:
          return {
            type: criteria.type,
            passed: true,
            score: 1.0,
            message: `Unknown check type '${criteria.type}' skipped`,
            details: {
              skipped: true,
            },
          };
      }
    } catch (error) {
      this.logger.error(`Validation check failed for ${criteria.type}:`, error);
      return {
        type: criteria.type,
        passed: false,
        message: `Check failed: ${error instanceof Error ? error.message : String(error)}`,
      };
    }
  }

  /**
   * Run LLM self-check validation
   *
   * TODO: IMPLEMENT LLM API INTEGRATION
   *
   * This is the ONLY validation type that can meaningfully assess LLM-generated content.
   * String-based checks (length, patterns, keywords) have been intentionally removed
   * because they don't correlate with output quality.
   *
   * Implementation requirements:
   * - LLM client instance (from semantic analyzer or external API)
   * - Validation prompt templates (quality rubrics, evaluation criteria)
   * - Structured output parsing (pass/fail with confidence scores)
   * - Confidence threshold enforcement
   *
   * Configuration path: config.analysis.semanticAnalysis.llmIntegration
   *
   * Example implementation approach:
   * 1. Format validation prompt with content and criteria
   * 2. Call LLM with structured output schema (JSON mode)
   * 3. Parse response: { passed: boolean, score: number, feedback: string }
   * 4. Apply confidence threshold from criteria.pass_threshold
   *
   * Current behavior: Gracefully skips when LLM not configured
   */
  /**
   * Validates LLM configuration parameters to ensure they are properly set
   *
   * This method performs comprehensive validation of LLM integration configuration:
   * - Verifies API key is present and not empty
   * - Ensures model is specified and not empty
   * - Validates maxTokens is within acceptable range (1 to 1,000,000)
   * - Checks temperature is within valid range (0 to 2)
   * - Validates endpoint URL format if provided
   *
   * @param llmConfig - The LLM integration configuration to validate
   * @returns null if valid, or an error message string if validation fails
   */
  private validateLLMConfig(llmConfig: LLMIntegrationConfig): string | null {
    if (!llmConfig.apiKey || llmConfig.apiKey.trim() === '') {
      return 'LLM API key is missing or empty';
    }

    if (!llmConfig.model || llmConfig.model.trim() === '') {
      return 'LLM model is missing or empty';
    }

    if (llmConfig.maxTokens !== undefined && (llmConfig.maxTokens <= 0 || llmConfig.maxTokens > 1000000)) {
      return 'LLM maxTokens must be between 1 and 1,000,000';
    }

    if (llmConfig.temperature !== undefined && (llmConfig.temperature < 0 || llmConfig.temperature > 2)) {
      return 'LLM temperature must be between 0 and 2';
    }

    // Validate endpoint URL format if present
    if (llmConfig.endpoint) {
      try {
        new URL(llmConfig.endpoint);
      } catch (e) {
        return `LLM endpoint is not a valid URL: ${llmConfig.endpoint}`;
      }
    }

    return null; // No validation errors
  }

  /**
   * Runs LLM-based self-check validation for the given criteria and context.
   *
   * This method performs LLM-based validation with comprehensive error handling:
   * - Validates LLM configuration parameters before execution
   * - Handles missing or invalid endpoint configurations
   * - Manages API key and model validation
   * - Provides appropriate feedback when LLM integration is disabled
   * - Implements graceful fallback when LLM services are unavailable
   *
   * @param criteria - The validation criteria to check
   * @param context - The validation context containing content and metadata
   * @returns A ValidationCheck result with pass/fail status and details
   */
  private async runLLMSelfCheck(
    criteria: GatePassCriteria,
    context: ValidationContext
  ): Promise<ValidationCheck> {
    // Check if LLM integration is configured and enabled
    const llmConfig = this.llmConfig;
    if (!llmConfig?.enabled) {
      this.logger.debug('[LLM GATE] LLM self-check skipped - LLM integration disabled in config');
      return {
        type: 'llm_self_check',
        passed: true, // Auto-pass when not configured
        score: 1.0,
        message:
          'LLM validation skipped (not configured - set analysis.semanticAnalysis.llmIntegration.enabled=true)',
        details: {
          skipped: true,
          reason: 'LLM integration disabled in config',
          configPath: 'config.analysis.semanticAnalysis.llmIntegration.enabled',
        },
      };
    }

    // Validate LLM configuration parameters
    const validationError = this.validateLLMConfig(llmConfig);
    if (validationError) {
      this.logger.warn(`[LLM GATE] LLM configuration validation failed: ${validationError}`);
      return {
        type: 'llm_self_check',
        passed: false,
        score: 0,
        message: `LLM configuration validation failed: ${validationError}`,
        details: {
          validationError,
          reason: 'Invalid LLM configuration parameters',
        },
      };
    }

    if (llmConfig.endpoint === undefined || llmConfig.endpoint === '') {
      this.logger.warn('[LLM GATE] LLM self-check skipped - no endpoint configured');
      return {
        type: 'llm_self_check',
        passed: false, // Changed to fail when no endpoint is configured
        score: 0,
        message: 'LLM validation failed (no endpoint configured)',
        details: {
          skipped: false,
          reason: 'No LLM endpoint configured',
          configPath: 'config.analysis.semanticAnalysis.llmIntegration.endpoint',
        },
      };
    }

    const prompt = this.buildSelfCheckPrompt(criteria, context);
    const threshold = criteria.pass_threshold ?? 0.7;

    try {
      const responseText = await this.callLLMSelfCheck(llmConfig, prompt);
      const parsed = this.parseSelfCheckResponse(responseText);

      const score = parsed.score ?? 0;
      const passed = parsed.passed ?? score >= threshold;

      return {
        type: 'llm_self_check',
        passed,
        score,
        message: parsed.feedback ?? 'LLM self-check completed',
        details: {
          threshold,
          raw: parsed.raw,
        },
      };
    } catch (error) {
      this.logger.error('[LLM GATE] LLM self-check failed:', error);
      return {
        type: 'llm_self_check',
        passed: false,
        message: `LLM self-check failed: ${error instanceof Error ? error.message : String(error)}`,
        details: {
          threshold,
        },
      };
    }
  }

  private getOrCreateRegex(pattern: string, flags: string = 'i'): RegExp {
    const cacheKey = `${pattern}:${flags}`;
    const cached = this.regexCache.get(cacheKey);
    if (cached) {
      return cached;
    }

    const regex = new RegExp(pattern, flags);
    this.regexCache.set(cacheKey, regex);
    return regex;
  }

  /**
   * Clears the regex cache to free up memory
   * Useful for long-running processes to prevent memory accumulation
   */
  clearRegexCache(): void {
    this.regexCache.clear();
  }

  private runContentCheck(criteria: GatePassCriteria, context: ValidationContext): ValidationCheck {
    const content = context.content ?? '';
    const messages: string[] = [];
    let passed = true;

    if (criteria.min_length !== undefined && content.length < criteria.min_length) {
      passed = false;
      messages.push(`Content too short: ${content.length} < ${criteria.min_length} characters`);
    }

    if (criteria.max_length !== undefined && content.length > criteria.max_length) {
      passed = false;
      messages.push(`Content too long: ${content.length} > ${criteria.max_length} characters`);
    }

    if (criteria.required_patterns && criteria.required_patterns.length > 0) {
      for (const pattern of criteria.required_patterns) {
        try {
          const regex = this.getOrCreateRegex(pattern, 'i');
          if (!regex.test(content)) {
            passed = false;
            messages.push(`Missing required pattern: ${pattern}`);
          }
        } catch {
          messages.push(`Invalid regex pattern: ${pattern}`);
        }
      }
    }

    if (criteria.forbidden_patterns && criteria.forbidden_patterns.length > 0) {
      for (const pattern of criteria.forbidden_patterns) {
        try {
          const regex = this.getOrCreateRegex(pattern, 'i');
          if (regex.test(content)) {
            passed = false;
            messages.push(`Contains forbidden pattern: ${pattern}`);
          }
        } catch {
          messages.push(`Invalid regex pattern: ${pattern}`);
        }
      }
    }

    return {
      type: 'content_check',
      passed,
      score: passed ? 1.0 : 0.0,
      message: messages.length > 0 ? messages.join('; ') : 'Content check passed',
    };
  }

  private runPatternCheck(criteria: GatePassCriteria, context: ValidationContext): ValidationCheck {
    const content = context.content ?? '';
    const messages: string[] = [];
    let passed = true;

    if (criteria.regex_patterns && criteria.regex_patterns.length > 0) {
      for (const pattern of criteria.regex_patterns) {
        try {
          const regex = this.getOrCreateRegex(pattern, 'i');
          if (!regex.test(content)) {
            passed = false;
            messages.push(`Missing regex pattern: ${pattern}`);
          }
        } catch {
          messages.push(`Invalid regex pattern: ${pattern}`);
        }
      }
    }

    if (criteria.keyword_count) {
      for (const [keyword, count] of Object.entries(criteria.keyword_count)) {
        try {
          const regex = this.getOrCreateRegex(keyword, 'gi');
          const found = (content.match(regex) || []).length;
          if (found < count) {
            passed = false;
            messages.push(`Keyword '${keyword}' appears ${found}/${count} times`);
          }
        } catch {
          messages.push(`Invalid keyword regex pattern: ${keyword}`);
        }
      }
    }

    return {
      type: 'pattern_check',
      passed,
      score: passed ? 1.0 : 0.0,
      message: messages.length > 0 ? messages.join('; ') : 'Pattern check passed',
    };
  }

  private buildSelfCheckPrompt(criteria: GatePassCriteria, context: ValidationContext): string {
    const metadata = JSON.stringify(context.metadata || {}, null, 2);
    const checks = context.metadata?.['checks'] || [];
    const executionContext =
      context.executionContext !== undefined
        ? JSON.stringify(context.executionContext, null, 2)
        : '{}';

    const defaultTemplate = `You are a strict quality gate for LLM outputs.
Evaluate the content and return JSON: {"passed": boolean, "score": number, "feedback": string}.
Score must be between 0 and 1. Use the "feedback" field for concise rationale.

Content:
{{content}}

Metadata:
{{metadata}}

ExecutionContext:
{{execution_context}}`;

    const promptTemplate = criteria.prompt_template || defaultTemplate;

    return promptTemplate
      .replace('{{content}}', context.content ?? '')
      .replace('{{metadata}}', metadata)
      .replace('{{execution_context}}', executionContext);
  }

  private async callLLMSelfCheck(llmConfig: LLMIntegrationConfig, prompt: string): Promise<string> {
    const endpoint = llmConfig.endpoint ?? 'https://api.openai.com/v1/chat/completions';
    const lower = endpoint.toLowerCase();

    if (lower.includes('api.anthropic.com')) {
      const response = await fetch(endpoint, {
        method: 'POST',
        headers: {
          Authorization: `Bearer ${llmConfig.apiKey ?? ''}`,
          'Content-Type': 'application/json',
          'anthropic-version': '2023-06-01',
        },
        body: JSON.stringify({
          model: llmConfig.model,
          max_tokens: llmConfig.maxTokens,
          temperature: llmConfig.temperature,
          messages: [{ role: 'user', content: prompt }],
        }),
      });

      if (!response.ok) {
        throw new Error(`Anthropic API error: ${response.status} ${response.statusText}`);
      }

      const data = (await response.json()) as { content?: Array<{ text: string }> };
      const content = data.content?.[0]?.text;
      if (content === undefined || content === '') {
        throw new Error('No content in Anthropic response');
      }
      return content;
    }

    const headers: Record<string, string> = {
      'Content-Type': 'application/json',
    };
    if (llmConfig.apiKey) {
      headers['Authorization'] = `Bearer ${llmConfig.apiKey}`;
    }

    const response = await fetch(endpoint, {
      method: 'POST',
      headers,
      body: JSON.stringify({
        model: llmConfig.model,
        messages: [
          {
            role: 'system',
            content:
              'You are a strict validator. Return only valid JSON with passed/score/feedback.',
          },
          {
            role: 'user',
            content: prompt,
          },
        ],
        max_tokens: llmConfig.maxTokens,
        temperature: llmConfig.temperature,
      }),
    });

    if (!response.ok) {
      throw new Error(`LLM API error: ${response.status} ${response.statusText}`);
    }

    const data = (await response.json()) as {
      choices?: Array<{ message?: { content?: string } }>;
    };
    const content = data.choices?.[0]?.message?.content;
    if (content === undefined || content === '') {
      throw new Error('No content in LLM response');
    }

    return content;
  }

  private parseSelfCheckResponse(responseText: string): {
    passed?: boolean;
    score?: number;
    feedback?: string;
    raw?: string;
  } {
    try {
      const parsed = JSON.parse(responseText);
      return {
        passed: typeof parsed.passed === 'boolean' ? parsed.passed : undefined,
        score: typeof parsed.score === 'number' ? parsed.score : undefined,
        feedback: typeof parsed.feedback === 'string' ? parsed.feedback : undefined,
        raw: responseText,
      };
    } catch {
      return {
        raw: responseText,
      };
    }
  }

  /**
   * Generate retry hints based on failed checks
   *
   * With string-based validation removed, hints now focus on:
   * 1. Gate-specific guidance (from gate definition)
   * 2. LLM self-check feedback (when implemented)
   * 3. Generic quality improvement suggestions
   */
  private generateRetryHints(gate: LightweightGateDefinition, checks: ValidationCheck[]): string[] {
    const hints: string[] = [];
    const failedChecks = checks.filter((check) => !check.passed);

    if (failedChecks.length === 0) {
      return hints;
    }

    // Add gate-specific guidance as a hint
    // Skip for inline gates - criteria already displayed prominently in "Inline Quality Criteria" section
    const isInlineGate = gate.name.includes('Inline Quality') || gate.id.startsWith('temp_');
    if (gate.retry_config?.improvement_hints) {
      hints.push(`Remember the ${gate.name} guidelines:\n${gate.guidance}`);
    }

    // Add LLM self-check specific hints (the only meaningful validation)
    for (const check of failedChecks) {
      if (check.type === 'llm_self_check') {
        hints.push('Review the quality criteria and improve content structure and depth');
        // When LLM validation is implemented, this would include specific feedback
        const detailParams = check.details as
          | { passed?: boolean; score?: number; feedback?: string }
          | undefined;
        if (detailParams?.feedback !== undefined) {
          hints.push(detailParams.feedback);
        }
      }
    }

    // Ensure we have at least one helpful hint
    if (hints.length === 0) {
      hints.push(`${gate.name} validation failed. Please review the requirements and try again.`);
    }

    return hints;
  }

  /**
   * Check if content should be retried based on validation results
   *
   * @param validationResults - Results from gate validation
   * @param currentAttempt - Current attempt number
   * @param maxAttempts - Maximum allowed attempts
   * @returns true if retry should be attempted
   */
  shouldRetry(
    validationResults: ValidationResult[],
    currentAttempt: number,
    maxAttempts: number = 3
  ): boolean {
    if (currentAttempt >= maxAttempts) {
      this.logger.debug('[GATE VALIDATOR] Max attempts reached, no retry');
      return false;
    }

    // Retry if any validation gate failed
    const shouldRetry = validationResults.some((result) => !result.valid);

    if (shouldRetry) {
      this.validationStats.retryRequests++;
      this.logger.debug('[GATE VALIDATOR] Retry recommended:', {
        currentAttempt,
        maxAttempts,
        failedGates: validationResults.filter((r) => !r.valid).map((r) => r.gateId),
      });
    }

    return shouldRetry;
  }

  /**
   * Update average validation time
   */
  private updateAverageValidationTime(): void {
    if (this.validationTimes.length > 0) {
      const sum = this.validationTimes.reduce((a, b) => a + b, 0);
      this.validationStats.averageValidationTime = sum / this.validationTimes.length;
    }

    // Keep only last 100 measurements for rolling average
    if (this.validationTimes.length > 100) {
      this.validationTimes = this.validationTimes.slice(-100);
    }
  }

  /**
   * Get validation statistics
   */
  getStatistics(): GateValidationStatistics {
    return { ...this.validationStats };
  }

  /**
   * Reset validation statistics
   */
  resetStatistics(): void {
    this.validationStats = {
      totalValidations: 0,
      successfulValidations: 0,
      failedValidations: 0,
      averageValidationTime: 0,
      retryRequests: 0,
    };
    this.validationTimes = [];
    this.logger.debug('[GATE VALIDATOR] Statistics reset');
  }
}

/**
 * Create a gate validator instance
 */
export function createGateValidator(
  logger: Logger,
  gateLoader: GateDefinitionProvider,
  llmConfig?: LLMIntegrationConfig
): GateValidator {
  return new GateValidator(logger, gateLoader, llmConfig);
}
