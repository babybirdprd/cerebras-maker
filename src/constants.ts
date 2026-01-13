import { GraphLink, GraphNode, Task, VoteCandidate } from './types';

export const MOCK_PLAN: Task[] = [
  {
    id: 'root',
    title: 'Implement Auth Service with JWT',
    status: 'active',
    depth: 0,
    details: {
      issues: ['#101: System Architecture', '#102: Security Review'],
      notes: 'Primary authentication subsystem for the platform.'
    },
    children: [
      {
        id: 't1',
        title: 'Define User Schema (Diesel)',
        status: 'completed',
        depth: 1,
        details: {
          issues: ['#205: User Table Migration'],
          snippet: `#[derive(Queryable, Selectable)]\n#[diesel(table_name = crate::schema::users)]\npub struct User {\n    pub id: i32,\n    pub username: String,\n    pub password_hash: String,\n}`
        },
        children: []
      },
      {
        id: 't2',
        title: 'Implement Token Generation',
        status: 'active',
        depth: 1,
        children: [
          { 
            id: 't2a', 
            title: 'Create RSA Keypair Util', 
            status: 'completed', 
            depth: 2,
            details: {
              issues: ['#210: Crypto Utils'],
              snippet: `pub fn load_keys() -> (RsaPrivateKey, RsaPublicKey) {\n    // Loads keys from .env or generates ephemeral pair\n    // ...\n}`
            }
          },
          { 
            id: 't2b', 
            title: 'Implement Claims Struct', 
            status: 'active', 
            depth: 2,
            details: {
              issues: ['#211: JWT Claims definition'],
              snippet: `#[derive(Debug, Serialize, Deserialize)]\npub struct Claims {\n    pub sub: String,\n    pub exp: usize,\n    pub role: UserRole,\n}`
            }
          },
          { 
            id: 't2c', 
            title: 'Sign Token Function', 
            status: 'pending', 
            depth: 2,
            details: {
              issues: ['#212: Signing Logic'],
              notes: 'Waiting for Claims struct implementation.'
            }
          }
        ]
      },
      {
        id: 't3',
        title: 'Middleware Integration (Axum)',
        status: 'pending',
        depth: 1,
        children: []
      }
    ]
  }
];

export const GRAPH_NODES: GraphNode[] = [
  { id: 'main', group: 1, val: 20, label: 'main.rs' },
  { id: 'auth_mod', group: 1, val: 15, label: 'auth/mod.rs' },
  { id: 'jwt_util', group: 1, val: 10, label: 'utils/jwt.rs' },
  { id: 'user_model', group: 2, val: 12, label: 'models/user.rs' },
  { id: 'db_pool', group: 4, val: 10, label: 'db/pool.rs' },
  { id: 'config', group: 3, val: 8, label: 'config.rs' },
  { id: 'routes', group: 1, val: 15, label: 'routes.rs' },
];

export const GRAPH_LINKS: GraphLink[] = [
  { source: 'main', target: 'config', value: 1 },
  { source: 'main', target: 'routes', value: 5 },
  { source: 'routes', target: 'auth_mod', value: 3 },
  { source: 'auth_mod', target: 'jwt_util', value: 5 },
  { source: 'auth_mod', target: 'user_model', value: 2 },
  { source: 'user_model', target: 'db_pool', value: 1 },
];

export const CANDIDATES: VoteCandidate[] = [
  {
    id: 1,
    snippet: `pub fn sign_token(claims: Claims) -> Result<String, Error> {\n    let key = get_secret()?;\n    encode(&Header::default(), &claims, &key)\n}`,
    score: 0.98,
    redFlags: [],
    status: 'accepted'
  },
  {
    id: 2,
    snippet: `pub fn sign_token(claims: Claims) -> String {\n    // Unsafe unwrap detected\n    let key = get_secret().unwrap();\n    encode(..., ..., &key).unwrap()\n}`,
    score: 0.45,
    redFlags: ['Panic Prone (unwrap)', 'Topological Cycle'],
    status: 'rejected'
  },
  {
    id: 3,
    snippet: `async fn sign_token(claims: Claims) -> Result<String, Error> {\n    // Incorrect async usage for blocking op\n    let key = fetch_remote_key().await?;\n    encode(..., ..., &key)\n}`,
    score: 0.72,
    redFlags: ['Layer Violation: Network call in Utility'],
    status: 'rejected'
  }
];

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

