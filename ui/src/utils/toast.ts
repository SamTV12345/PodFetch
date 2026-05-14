/**
 * Drop-in shim that maps notistack's `enqueueSnackbar(message, { variant })`
 * API to sonner's `toast.success / .error / .warning / .info` so the
 * existing 20+ call sites keep working unchanged through the migration.
 *
 * Notistack -> sonner key differences silently smoothed over:
 * - notistack's `variant: 'default'` maps to sonner's plain `toast()` (no icon).
 * - notistack's `autoHideDuration` becomes sonner's `duration` (same ms unit).
 * - notistack accepted `key` for de-duping; sonner uses `id`.
 *
 * Anything notistack-specific that's not used in this codebase (action
 * buttons, anchor origin, custom variants, ...) is intentionally
 * dropped - none of the call sites pass them.
 */
import { toast, type ExternalToast } from "sonner"

export type SnackbarVariant = "default" | "success" | "error" | "warning" | "info"

export interface SnackbarOptions extends ExternalToast {
    variant?: SnackbarVariant
    autoHideDuration?: number
    key?: string | number
}

export function enqueueSnackbar(
    message: React.ReactNode,
    options: SnackbarOptions = {}
): string | number {
    const { variant = "default", autoHideDuration, key, ...rest } = options
    const data: ExternalToast = {
        ...rest,
        ...(autoHideDuration !== undefined ? { duration: autoHideDuration } : {}),
        ...(key !== undefined ? { id: key } : {}),
    }
    switch (variant) {
        case "success":
            return toast.success(message, data)
        case "error":
            return toast.error(message, data)
        case "warning":
            return toast.warning(message, data)
        case "info":
            return toast.info(message, data)
        default:
            return toast(message, data)
    }
}

/** Hook compat: sonner has no hook equivalent - return the same enqueue fn. */
export function useSnackbar() {
    return { enqueueSnackbar }
}
