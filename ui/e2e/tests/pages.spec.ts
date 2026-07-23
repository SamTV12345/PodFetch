import { expect, test } from '@playwright/test'

// Coverage for the routes the other suites don't touch: the home dashboard,
// the waiting list, the rescan/gpodder/mopidy settings tabs, tags, discover,
// the profile page and the user-administration tables. The e2e server runs
// without auth, which provisions a standard admin user, so the admin-only
// pages render their real content here.

test('home dashboard renders its episode rails', async ({ page }) => {
    await page.goto('/ui/home/view')

    await expect(page.getByRole('heading', { name: 'Recently listened' })).toBeVisible()
    await expect(page.getByRole('heading', { name: 'Latest episodes' })).toBeVisible()
})

test('waiting list page renders its empty state', async ({ page }) => {
    await page.goto('/ui/waiting-list')

    await expect(page.getByRole('heading', { name: 'Waiting list' })).toBeVisible()
    await expect(
        page.getByText('Nothing queued yet. Pick episodes from your inbox.')
    ).toBeVisible()
})

test('rescan settings can re-apply the filename format', async ({ page }) => {
    await page.goto('/ui/settings/rescan')

    const applyFilenames = page.getByText('Re-apply filename and directory format')
    await expect(applyFilenames).toBeVisible()
    await applyFilenames.click()

    await page.getByRole('button', { name: 'Rescan audio files' }).click()
    await expect(page.getByText('Rescan done')).toBeVisible()
})

test('tags can be created and deleted', async ({ page }) => {
    await page.goto('/ui/tags')
    await expect(page.getByRole('heading', { name: 'Tags' })).toBeVisible()

    await page.getByPlaceholder('Add tag').fill('E2E Coverage Tag')
    await page.getByRole('button', { name: 'Add', exact: true }).click()

    // The new tag renders as an editable row; its input carries the name.
    await expect(page.getByRole('textbox').last()).toHaveValue('E2E Coverage Tag')

    // Clean up so later suites see no tags.
    await page.getByRole('button', { name: 'Delete' }).click()
    await expect(
        page.getByText('No tags created yet. Use the form above to add one.')
    ).toBeVisible()
})

test('gpodder settings render the available-podcasts table', async ({ page }) => {
    await page.goto('/ui/settings/gpodder')

    await expect(page.getByRole('columnheader', { name: 'Device' })).toBeVisible()
})

test('mopidy settings render the server list', async ({ page }) => {
    await page.goto('/ui/settings/mopidy')

    await expect(page.getByText('No Mopidy servers configured yet.')).toBeVisible()
})

test('discover page renders its tabs', async ({ page }) => {
    await page.goto('/ui/discover')

    await expect(page.getByRole('heading', { name: 'Discover' })).toBeVisible()
    await expect(page.getByRole('tab', { name: 'For you' })).toBeVisible()
})

test('profile page renders the account form', async ({ page }) => {
    await page.goto('/ui/profile')

    await expect(page.getByRole('heading', { name: 'Profile' })).toBeVisible()
    await expect(page.getByText('API key', { exact: true })).toBeVisible()
})

test('user administration lists users', async ({ page }) => {
    await page.goto('/ui/administration/users')

    await expect(page.getByRole('columnheader', { name: 'Username' })).toBeVisible()
})

test('invites administration renders and offers a create action', async ({ page }) => {
    await page.goto('/ui/administration/invites')

    await expect(page.getByRole('columnheader', { name: 'Role' })).toBeVisible()
    await expect(page.getByRole('button', { name: 'Add new' })).toBeVisible()
})
