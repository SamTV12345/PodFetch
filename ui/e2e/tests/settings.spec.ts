import { expect, test } from '@playwright/test'

test('retention settings persist across reloads', async ({ page }) => {
    await page.goto('/ui/settings/retention')

    const daysToKeep = page.locator('#days-to-keep')
    await expect(daysToKeep).toBeVisible()
    const original = await daysToKeep.inputValue()

    await daysToKeep.fill('31')
    await page.getByRole('button', { name: 'Save' }).click()
    await expect(page.getByText('Settings saved')).toBeVisible()

    await page.reload()
    await expect(daysToKeep).toHaveValue('31')

    // Restore the original value so later suites see pristine settings.
    await daysToKeep.fill(original)
    await page.getByRole('button', { name: 'Save' }).click()
    await expect(page.getByText('Settings saved')).toBeVisible()
})

test('naming settings page renders its configuration controls', async ({ page }) => {
    await page.goto('/ui/settings/naming')

    await expect(page.getByText('Colon replacement')).toBeVisible()
    await expect(page.getByRole('button', { name: 'Save' })).toBeVisible()
})

test('opml export downloads a feed list', async ({ page }) => {
    await page.goto('/ui/settings/opml')

    const downloadPromise = page.waitForEvent('download')
    await page.getByRole('button', { name: 'Download' }).first().click()
    const download = await downloadPromise

    expect(download.suggestedFilename()).toBe('podcast_local.opml')
})

test('theme selection applies dark mode and survives reloads', async ({ page }) => {
    await page.goto('/ui/podcasts')

    const isDark = () => page.evaluate(() => document.documentElement.classList.contains('dark'))

    await page.getByRole('button', { name: 'Dark', exact: true }).click()
    expect(await isDark()).toBe(true)

    await page.reload()
    expect(await isDark()).toBe(true)

    await page.getByRole('button', { name: 'Light', exact: true }).click()
    expect(await isDark()).toBe(false)

    // Back to the default system preference.
    await page.getByRole('button', { name: 'System', exact: true }).click()
})

test('language can be switched and translates the ui', async ({ page }) => {
    await page.goto('/ui/podcasts/search')
    await expect(page.getByRole('heading', { name: 'Search episodes' })).toBeVisible()

    await page.locator('.i18n-dropdown').click()
    await page.getByRole('option', { name: 'Deutsch' }).click()
    await expect(page.getByRole('heading', { name: 'Episoden suchen' })).toBeVisible()

    // And back to English for the rest of the suite.
    await page.locator('.i18n-dropdown').click()
    await page.getByRole('option', { name: 'English' }).click()
    await expect(page.getByRole('heading', { name: 'Search episodes' })).toBeVisible()
})

test('notifications popover opens and reports its state', async ({ page }) => {
    await page.goto('/ui/podcasts')

    await page.getByRole('button', { name: 'Open notifications' }).click()
    // Feed refreshes create real notifications, so accept either the unread
    // header or the empty state.
    await expect(page.getByText(/unread notifications|No notifications/).first()).toBeVisible()
})
