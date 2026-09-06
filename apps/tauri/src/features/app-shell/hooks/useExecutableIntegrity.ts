import { useCallback, useReducer, useRef } from 'react';
import { scanExecutableIntegrity } from '../../../lib/tauri';
import {
  idleAsyncOperation,
  reduceAsyncOperation,
  operationData,
  type AsyncOperationState,
} from '../../../model/async';
import type { IntegrityScanResponse } from '../../../types/workflow';

export const defaultIntegrityTools = ['node', 'bun', 'cargo', 'rustc', 'git', 'java', 'gradle'];

export type ExecutableIntegrityState = {
  operation: AsyncOperationState<IntegrityScanResponse>;
  result: IntegrityScanResponse | undefined;
  busy: boolean;
  scan: (tools: string[]) => Promise<void>;
};

export function useExecutableIntegrity(): ExecutableIntegrityState {
  const [operation, dispatch] = useReducer(
    reduceAsyncOperation<IntegrityScanResponse>,
    idleAsyncOperation<IntegrityScanResponse>(),
  );
  const requestRef = useRef(0);

  const scan = useCallback(async (tools: string[]) => {
    const requestId = ++requestRef.current;
    dispatch({ type: 'start', requestId });

    try {
      const response = await scanExecutableIntegrity({ tools: [...tools] });
      dispatch({ type: 'success', requestId, data: response });
    } catch (error) {
      dispatch({ type: 'error', requestId, error: String(error) });
    }
  }, []);

  return {
    operation,
    result: operationData(operation),
    busy: operation.status === 'loading',
    scan,
  };
}
