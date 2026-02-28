import { describe, it, expect, vi, beforeEach } from 'vitest';

// Mock fetch globally
const fetchMock = vi.fn();
vi.stubGlobal('fetch', fetchMock);

// Import after mocking
const { api } = await import('./api.ts');

describe('api.request', () => {
  beforeEach(() => {
    fetchMock.mockReset();
  });

  it('should call fetch with correct URL and headers', async () => {
    fetchMock.mockResolvedValue({
      ok: true,
      status: 200,
      json: () => Promise.resolve([]),
    });

    const result = await api.listSkills();
    expect(fetchMock).toHaveBeenCalledWith('/api/skills', expect.objectContaining({
      headers: expect.objectContaining({ 'Content-Type': 'application/json' }),
    }));
    expect(result).toEqual([]);
  });

  it('should throw on non-ok response', async () => {
    fetchMock.mockResolvedValue({
      ok: false,
      status: 404,
      text: () => Promise.resolve('Not Found'),
    });

    await expect(api.getSkill('nonexistent')).rejects.toThrow('404: Not Found');
  });

  it('should return null for 204 responses', async () => {
    fetchMock.mockResolvedValue({
      ok: true,
      status: 204,
    });

    const result = await api.deleteSession('test-id');
    expect(result).toBeNull();
  });

  it('should POST with JSON body for createSession', async () => {
    fetchMock.mockResolvedValue({
      ok: true,
      status: 200,
      json: () => Promise.resolve({ id: 'abc', target: 'http://test.com', mode: 'interactive' }),
    });

    const result = await api.createSession('http://test.com', 'interactive');
    expect(fetchMock).toHaveBeenCalledWith('/api/sessions', expect.objectContaining({
      method: 'POST',
      body: JSON.stringify({ target: 'http://test.com', mode: 'interactive' }),
    }));
    expect(result.id).toBe('abc');
  });
});
