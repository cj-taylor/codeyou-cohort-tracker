import { state } from '../state.js';

describe('state', () => {
  test('has correct initial values', () => {
    expect(state.currentClassId).toBeNull();
    expect(state.demoMode).toBe(true);
    expect(state.previousSectionModal).toBeNull();
    expect(state.studentActivityData).toEqual([]);
    expect(state.allClasses).toEqual([]);
    expect(state.API_BASE).toBe('');
  });
  
  test('can be modified', () => {
    state.currentClassId = 'test-class';
    expect(state.currentClassId).toBe('test-class');
    
    state.demoMode = false;
    expect(state.demoMode).toBe(false);
    
    // Reset for other tests
    state.currentClassId = null;
    state.demoMode = true;
  });
});
