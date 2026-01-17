// Cerebras-MAKER Constants
// Only contains legitimate configuration - no mock data

// Default agent configurations
export const DEFAULT_AGENT_CONFIG = {
  interrogator: {
    provider: 'anthropic' as const,
    model: 'claude-sonnet-4-20250514',
    temperature: 0.3,
  },
  architect: {
    provider: 'openai' as const,
    model: 'gpt-4o',
    temperature: 0.5,
  },
  orchestrator: {
    provider: 'cerebras' as const,
    model: 'llama-4-scout-17b-16e-instruct',
    temperature: 0.7,
  },
  coder: {
    provider: 'cerebras' as const,
    model: 'llama-4-scout-17b-16e-instruct',
    temperature: 0.1,
  },
  reviewer: {
    provider: 'anthropic' as const,
    model: 'claude-sonnet-4-20250514',
    temperature: 0.3,
  },
  tester: {
    provider: 'openai' as const,
    model: 'gpt-4o-mini',
    temperature: 0.2,
  },
};

