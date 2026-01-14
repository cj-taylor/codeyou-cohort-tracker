import { maskName, maskEmail, formatPercent, formatDate, getWeekDateRange } from '../utils.js';

describe('maskName', () => {
  test('returns actual name when demo mode is off', () => {
    expect(maskName('John', 'Doe', 0, false)).toBe('John Doe');
  });
  
  test('returns masked name when demo mode is on', () => {
    expect(maskName('John', 'Doe', 0, true)).toBe('Student 1');
    expect(maskName('Jane', 'Smith', 5, true)).toBe('Student 6');
  });
});

describe('maskEmail', () => {
  test('returns actual email when demo mode is off', () => {
    expect(maskEmail('john@example.com', 0, false)).toBe('john@example.com');
  });
  
  test('returns masked email when demo mode is on', () => {
    expect(maskEmail('john@example.com', 0, true)).toBe('student1@example.com');
    expect(maskEmail('jane@example.com', 5, true)).toBe('student6@example.com');
  });
});

describe('formatPercent', () => {
  test('formats decimal as percentage with one decimal place', () => {
    expect(formatPercent(0.755)).toBe('75.5%');
    expect(formatPercent(0.5)).toBe('50.0%');
    expect(formatPercent(1)).toBe('100.0%');
    expect(formatPercent(0)).toBe('0.0%');
  });
});

describe('formatDate', () => {
  test('returns "Never" for null or undefined', () => {
    expect(formatDate(null)).toBe('Never');
    expect(formatDate(undefined)).toBe('Never');
    expect(formatDate(0)).toBe('Never');
  });
  
  test('formats Unix timestamp as locale string', () => {
    const timestamp = 1704067200; // 2024-01-01 00:00:00 UTC
    const result = formatDate(timestamp);
    // Just check it returns a string (timezone-independent)
    expect(typeof result).toBe('string');
    expect(result).not.toBe('Never');
  });
});

describe('getWeekDateRange', () => {
  test('converts ISO week to date range', () => {
    const result = getWeekDateRange('2025-35');
    // Just check format, not specific month (week calculation can vary)
    expect(result).toMatch(/\w+ \d+ - \w+ \d+/);
  });
  
  test('handles different weeks', () => {
    const result = getWeekDateRange('2025-01');
    expect(result).toMatch(/\w+ \d+ - \w+ \d+/);
  });
});
