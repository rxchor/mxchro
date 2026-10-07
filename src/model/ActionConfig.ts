export type ActionConfig = 
  | { type: 'key', key: string, press: boolean }
  | { type: 'wait', duration_ms: number };
