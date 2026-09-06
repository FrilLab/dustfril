export type AsyncOperationStatus =
  | 'idle'
  | 'loading'
  | 'success'
  | 'partial'
  | 'unsupported'
  | 'empty'
  | 'error';

export type AsyncOperationState<T> =
  | { status: 'idle'; requestId: number }
  | { status: 'loading'; requestId: number }
  | { status: 'success'; requestId: number; data: T }
  | { status: 'partial'; requestId: number; data: T; warnings: string[] }
  | { status: 'unsupported'; requestId: number; reason: string }
  | { status: 'empty'; requestId: number }
  | { status: 'error'; requestId: number; error: string; previous?: T };

export type AsyncOperationAction<T> =
  | { type: 'start'; requestId: number }
  | { type: 'success'; requestId: number; data: T; warnings?: string[] }
  | { type: 'unsupported'; requestId: number; reason: string }
  | { type: 'empty'; requestId: number }
  | { type: 'error'; requestId: number; error: string }
  | { type: 'invalidate'; requestId: number };

export function idleAsyncOperation<T>(requestId = 0): AsyncOperationState<T> {
  return { status: 'idle', requestId };
}

export function reduceAsyncOperation<T>(
  state: AsyncOperationState<T>,
  action: AsyncOperationAction<T>,
): AsyncOperationState<T> {
  if (action.requestId < state.requestId) {
    return state;
  }

  switch (action.type) {
    case 'start':
      return { status: 'loading', requestId: action.requestId };
    case 'success':
      return action.warnings?.length
        ? {
            status: 'partial',
            requestId: action.requestId,
            data: action.data,
            warnings: action.warnings,
          }
        : { status: 'success', requestId: action.requestId, data: action.data };
    case 'unsupported':
      return { status: 'unsupported', requestId: action.requestId, reason: action.reason };
    case 'empty':
      return { status: 'empty', requestId: action.requestId };
    case 'error':
      return {
        status: 'error',
        requestId: action.requestId,
        error: action.error,
        previous: operationData(state),
      };
    case 'invalidate':
      return { status: 'idle', requestId: action.requestId };
  }
}

export function operationData<T>(state: AsyncOperationState<T>): T | undefined {
  return 'data' in state ? state.data : state.status === 'error' ? state.previous : undefined;
}
