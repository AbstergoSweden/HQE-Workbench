export function createMockGateGuidanceRenderer() {
  return {
    renderGuidance: () => Promise.resolve('Mock guidance'),
  };
}