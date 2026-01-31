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
 */
export class GateValidator {
  private logger: Logger;
  private gateLoader: GateDefinitionProvider;
  private llmConfig: LLMIntegrationConfig | undefined;
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
      const checks: ValidationCheck[] = [];
      let llmValidationUsed = false;

      if (gate.pass_criteria) {
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
    const results: ValidationResult[] = [];

    for (const gateId of gateIds) {
      const result = await this.validateGate(gateId, context);
      if (result) {
        results.push(result);

        // Update statistics based on result
        if (result.passed) {
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
  private async runLLMSelfCheck(
    criteria: GatePassCriteria,
    context: ValidationContext
  ): Promise<ValidationCheck> {
    // Check if LLM integration is configured and enabled
    const llmConfig = this.llmConfig;
    if (llmConfig?.enabled !== true) {
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

    if (llmConfig.endpoint === undefined || llmConfig.endpoint === '') {
      this.logger.warn('[LLM GATE] LLM self-check skipped - no endpoint configured');
      return {
        type: 'llm_self_check',
        passed: true,
        score: 1.0,
        message: 'LLM validation skipped (no endpoint configured)',
        details: {
          skipped: true,
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
          const regex = new RegExp(pattern, 'i');
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
          const regex = new RegExp(pattern, 'i');
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
          const regex = new RegExp(pattern, 'i');
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
        const found = (content.match(new RegExp(keyword, 'gi')) || []).length;
        if (found < count) {
          passed = false;
          messages.push(`Keyword '${keyword}' appears ${found}/${count} times`);
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
    const metadata = context.metadata ? JSON.stringify(context.metadata, null, 2) : '{}';
    const executionContext = context.executionContext
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

    const template = criteria.prompt_template?.trim().length
      ? criteria.prompt_template
      : defaultTemplate;

    return template
      .replace('{{content}}', context.content ?? '')
      .replace('{{metadata}}', metadata)
      .replace('{{execution_context}}', executionContext);
  }

  private async callLLMSelfCheck(
    llmConfig: LLMIntegrationConfig,
    prompt: string
  ): Promise<string> {
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

      const data = (await response.json()) as any;
      const content = data.content?.[0]?.text;
      if (!content) {
        throw new Error('No content in Anthropic response');
      }
      return content;
    }

    const headers: Record<string, string> = {
      'Content-Type': 'application/json',
    };
    if (llmConfig.apiKey) {
      headers.Authorization = `Bearer ${llmConfig.apiKey}`;
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

    const data = (await response.json()) as any;
    const content = data.choices?.[0]?.message?.content;
    if (!content) {
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
    const isInlineGate = gate.name?.includes('Inline Quality') || gate.id?.startsWith('temp_');
    if (gate.guidance && !isInlineGate) {
      hints.push(`Remember the ${gate.name} guidelines:\n${gate.guidance}`);
    }

    // Add LLM self-check specific hints (the only meaningful validation)
    for (const check of failedChecks) {
      if (check.type === 'llm_self_check') {
        hints.push('Review the quality criteria and improve content structure and depth');
        // When LLM validation is implemented, this would include specific feedback
        if (check.details?.['feedback'] !== undefined) {
          hints.push(check.details['feedback'] as string);
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
