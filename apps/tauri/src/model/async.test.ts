import { describe, expect, it } from 'vitest';
import {
  idleAsyncOperation,
  operationData,
  reduceAsyncOperation,
  type AsyncOperationState,
} from './async';

describe('async operation state', () => {
  it('keeps primary data and exposes auxiliary warnings as partial success', () => {
    const state = reduceAsyncOperation(
      reduceAsyncOperation(idleAsyncOperation<string>(), { type: 'start', requestId: 1 }),
      {
        type: 'success',
        requestId: 1,
        data: 'analysis',
        warnings: ['History could not be recorded.'],
      },
    );

    expect(state).toEqual({
      status: 'partial',
      requestId: 1,
      data: 'analysis',
      warnings: ['History could not be recorded.'],
    });
  });

  it('ignores stale responses after a newer request starts', () => {
    const loading = reduceAsyncOperation(
      reduceAsyncOperation(idleAsyncOperation<string>(), { type: 'start', requestId: 1 }),
      { type: 'start', requestId: 2 },
    );
    const state = reduceAsyncOperation(loading, {
      type: 'success',
      requestId: 1,
      data: 'stale result',
    });

    expect(state).toEqual(loading);
  });

  it('preserves the previous result when a refresh fails', () => {
    const previous: AsyncOperationState<string> = {
      status: 'success',
      requestId: 1,
      data: 'previous result',
    };
    const state = reduceAsyncOperation(previous, {
      type: 'error',
      requestId: 2,
      error: 'refresh failed',
    });

    expect(state).toEqual({
      status: 'error',
      requestId: 2,
      error: 'refresh failed',
      previous: 'previous result',
    });
    expect(operationData(state)).toBe('previous result');
  });

  it('invalidates a result when the workspace changes', () => {
    const state = reduceAsyncOperation(
      {
        status: 'success',
        requestId: 4,
        data: 'old workspace',
      },
      { type: 'invalidate', requestId: 5 },
    );

    expect(state).toEqual({ status: 'idle', requestId: 5 });
  });
});
