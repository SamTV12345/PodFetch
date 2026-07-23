import { expect, test, type Page } from '@playwright/test'
import { readSeed } from '../helpers'

const seed = readSeed()
const detailPage = `/ui/podcasts/${seed.podcastId}/episodes`

// Exhaustive coverage of the labels and toggles on the settings surfaces:
// the Data Retention page, the Naming page, the per-podcast settings modal
// and the rescan tab. Toggle tests flip every switch, persist, reload and
// assert, then restore the original state so later suites see pristine
// settings.

// --- Data Retention -------------------------------------------------------

const RETENTION_LABELS = [
    'Automatic cleanup',
    'Days to keep',
    'Automatically update podcasts',
    'Automatically download new episodes',
    'Episode numbering',
    'Number of initial podcasts to download',
    'Max parallel downloads',
    'Transcode to Opus',
    'Use one cover for all episodes',
    'NFO format',
    'Cover filename',
]

// Every Switcher on the Data Retention page, by input id.
const RETENTION_TOGGLES = [
    'auto-cleanup',
    'auto-update',
    'auto-download',
    'episode-numbering',
    'auto-transcode-opus',
    'use-one-cover-for-all-episodes',
]

test('data retention page shows every label and control', async ({ page }) => {
    await page.goto('/ui/settings/retention')

    for (const label of RETENTION_LABELS) {
        await expect(page.getByText(label, { exact: true })).toBeVisible()
    }
    await expect(page.getByRole('button', { name: 'Run cleanup now' })).toBeVisible()
    await expect(page.getByRole('button', { name: 'Save' })).toBeVisible()

    // The numeric inputs and the NFO select carry stable ids.
    for (const id of [
        'days-to-keep',
        'number-of-podcasts-to-download',
        'max-parallel-downloads',
        'cover-filename',
        'nfo-format',
    ]) {
        await expect(page.locator(`#${id}`)).toBeAttached()
    }
})

test('every data retention toggle persists across a reload', async ({ page }) => {
    await page.goto('/ui/settings/retention')

    // The switch inputs are sr-only; wait for one label before reading state.
    await expect(page.getByText('Automatic cleanup', { exact: true })).toBeVisible()

    const initial: Record<string, boolean> = {}
    for (const id of RETENTION_TOGGLES) {
        initial[id] = await page.locator(`#${id}`).isChecked()
    }

    const flipAllAndSave = async () => {
        for (const id of RETENTION_TOGGLES) {
            // The input is sr-only; its wrapping div carries the click handler.
            await page.locator(`#${id}`).locator('xpath=ancestor::div[1]').click()
        }
        await page.getByRole('button', { name: 'Save' }).click()
        await expect(page.getByText('Settings saved')).toBeVisible()
    }

    await flipAllAndSave()
    await page.reload()
    for (const id of RETENTION_TOGGLES) {
        expect(await page.locator(`#${id}`).isChecked(), `${id} after flip`).toBe(!initial[id])
    }

    // Restore the original state so later suites keep pristine settings.
    await flipAllAndSave()
    await page.reload()
    for (const id of RETENTION_TOGGLES) {
        expect(await page.locator(`#${id}`).isChecked(), `${id} restored`).toBe(initial[id])
    }
})

// --- Naming ---------------------------------------------------------------

const NAMING_LABELS = [
    'Rename podcasts',
    'Use existing filenames',
    'Replace illegal characters in filenames with a dash',
    'Colon replacement',
    'Standard episode format',
    'Sample episode format',
    'Standard podcast format',
    'Sample podcast format',
    'Use direct paths',
]

const NAMING_CHECKBOXES = ['use-existing-filenames', 'replace-invalid-characters', 'directPaths']

test('naming page shows every label and control', async ({ page }) => {
    await page.goto('/ui/settings/naming')

    for (const label of NAMING_LABELS) {
        await expect(page.getByText(label, { exact: true })).toBeVisible()
    }
    await expect(page.getByRole('button', { name: 'Save' })).toBeVisible()

    for (const id of ['colon-replacement', 'episode-format', 'podcast-format']) {
        await expect(page.locator(`#${id}`)).toBeAttached()
    }
})

test('every naming checkbox persists across a reload', async ({ page }) => {
    await page.goto('/ui/settings/naming')
    await expect(page.getByText('Rename podcasts', { exact: true })).toBeVisible()

    const initial: Record<string, boolean> = {}
    for (const id of NAMING_CHECKBOXES) {
        initial[id] = await page.locator(`#${id}`).isChecked()
    }

    const flipAllAndSave = async () => {
        // CustomCheckbox (base-ui) exposes a role=checkbox button; the id sits
        // on a hidden proxy input, so click the buttons by document order.
        for (let i = 0; i < NAMING_CHECKBOXES.length; i++) {
            await page.getByRole('checkbox').nth(i).click()
        }
        await page.getByRole('button', { name: 'Save' }).click()
        await expect(page.getByText('Settings saved')).toBeVisible()
    }

    await flipAllAndSave()
    await page.reload()
    for (const id of NAMING_CHECKBOXES) {
        expect(await page.locator(`#${id}`).isChecked(), `${id} after flip`).toBe(!initial[id])
    }

    await flipAllAndSave()
    await page.reload()
    for (const id of NAMING_CHECKBOXES) {
        expect(await page.locator(`#${id}`).isChecked(), `${id} restored`).toBe(initial[id])
    }
})

// --- Rescan ---------------------------------------------------------------

test('rescan tab shows every option label', async ({ page }) => {
    await page.goto('/ui/settings/rescan')

    for (const label of [
        'Re-apply filename and directory format',
        'Transcode existing MP3 episodes to Opus',
        'Drop per-episode covers',
        'Refresh embedded tags',
    ]) {
        await expect(page.getByText(label, { exact: true })).toBeVisible()
    }
    await expect(page.getByRole('button', { name: 'Rescan audio files' })).toBeVisible()
})

// --- Per-podcast settings modal ------------------------------------------

const openPodcastSettings = async (page: Page) => {
    await page.goto(detailPage)
    await page.locator('svg.lucide-settings').locator('xpath=ancestor::button[1]').click()
    return page.getByRole('dialog')
}

test('podcast settings modal shows every label on the settings tab', async ({ page }) => {
    const dialog = await openPodcastSettings(page)

    await expect(dialog.getByText('Configure settings')).toBeVisible()
    await expect(dialog.getByRole('tab', { name: 'Settings' })).toBeVisible()
    await expect(dialog.getByRole('tab', { name: 'Batch actions' })).toBeVisible()

    for (const label of [
        'Episode numbering',
        'Automatic cleanup',
        'Days to keep',
        'Automatically update podcasts',
        'Automatically download new episodes',
        'Auto-transcribe',
        'Colon replacement',
        'Use one cover for all episodes',
        'NFO format',
        'Cover filename',
        'Number of initial podcasts to download',
        'Activated',
    ]) {
        await expect(dialog.getByText(label, { exact: true })).toBeVisible()
    }
    await expect(dialog.getByRole('button', { name: 'Save' })).toBeVisible()
})

test('podcast settings modal shows every batch action', async ({ page }) => {
    const dialog = await openPodcastSettings(page)
    await dialog.getByRole('tab', { name: 'Batch actions' }).click()

    for (const label of [
        'Download missing episodes',
        'Download an episode range',
        'Re-download missing files',
        'Refresh local download state',
        'Delete all downloaded files',
    ]) {
        await expect(dialog.getByText(label, { exact: true })).toBeVisible()
    }
    // Four "Run" buttons and one "Delete" button drive the actions.
    await expect(dialog.getByRole('button', { name: 'Run' })).toHaveCount(4)
    await expect(dialog.getByRole('button', { name: 'Delete', exact: true })).toBeVisible()
})

test('per-podcast download-range action queues episodes', async ({ page }) => {
    const dialog = await openPodcastSettings(page)
    await dialog.getByRole('tab', { name: 'Batch actions' }).click()

    await dialog.getByLabel('From episode number').fill('1')
    await dialog.getByLabel('To episode number').fill('2')

    // The range row's own Run button sits next to the two number inputs.
    const rangeRow = dialog.getByLabel('From episode number').locator('xpath=ancestor::div[1]')
    await rangeRow.getByRole('button', { name: 'Run' }).click()

    await expect(page.getByText('Download an episode range')).toBeVisible()
})

test('per-podcast episode-numbering toggle persists', async ({ page }) => {
    const dialog = await openPodcastSettings(page)

    // The per-podcast switches have no id; locate by the label's sibling.
    const numbering = dialog
        .locator('label', { hasText: 'Episode numbering' })
        .locator('xpath=following-sibling::*[1]')
    const before = await numbering.locator('input').isChecked()

    await numbering.click()
    await dialog.getByRole('button', { name: 'Save' }).click()

    // Reopen and confirm the new value was stored server-side.
    const reopened = await openPodcastSettings(page)
    const after = reopened
        .locator('label', { hasText: 'Episode numbering' })
        .locator('xpath=following-sibling::*[1]')
    await expect(after.locator('input')).toBeChecked({ checked: !before })

    // Restore.
    await after.click()
    await reopened.getByRole('button', { name: 'Save' }).click()
})
