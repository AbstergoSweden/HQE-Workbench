export function createMockGateValidator() {
  return {
    validateGate: () => Promise.resolve(null),
    validateGates: () => Promise.resolve([]),
    shouldRetry: () => false,
  };
}