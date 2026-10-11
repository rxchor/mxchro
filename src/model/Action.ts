export type Action = 
  | { type: 'key', key: string, press: boolean }
  | { type: 'wait', duration_ms: number };
