import { describe, it, expect } from 'vitest';
import { getTheme, setTheme, toggleTheme } from './theme.ts';

describe('theme store', () => {
  it('should default to dark theme', () => {
    localStorage.removeItem('dalang-theme');
    expect(getTheme()).toBe('dark');
  });

  it('should persist and read theme', () => {
    setTheme('light');
    expect(getTheme()).toBe('light');
    expect(document.documentElement.getAttribute('data-theme')).toBe('light');

    setTheme('dark');
    expect(getTheme()).toBe('dark');
    expect(document.documentElement.getAttribute('data-theme')).toBe('dark');
  });

  it('should toggle between themes', () => {
    setTheme('dark');
    const next = toggleTheme();
    expect(next).toBe('light');
    expect(getTheme()).toBe('light');

    const next2 = toggleTheme();
    expect(next2).toBe('dark');
    expect(getTheme()).toBe('dark');
  });
});
