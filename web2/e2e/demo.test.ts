import { expect, test } from '@playwright/test';

test.describe('dashboard static build', () => {
	test('overview page shows war room heading', async ({ page }) => {
		await page.goto('/dashboard');
		await expect(page.getByRole('heading', { name: 'Operational Pulse' })).toBeVisible();
		await expect(page.getByText('DALANG WAR ROOM')).toBeVisible();
	});

	test('chat page shows console heading', async ({ page }) => {
		await page.goto('/dashboard/chat');
		await expect(page.getByRole('heading', { name: 'Interactive Console' })).toBeVisible();
	});

	test('skills catalog heading', async ({ page }) => {
		await page.goto('/dashboard/skills');
		await expect(page.getByRole('heading', { name: 'Skill Catalog' })).toBeVisible();
	});
});
