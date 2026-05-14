import { FC, useEffect, useRef, useState } from "react"
import {
    Dialog,
    DialogContent,
    DialogDescription,
    DialogHeader,
    DialogTitle,
} from "@/components/ui/dialog"
import { useTranslation } from "react-i18next"
import { enqueueSnackbar } from "notistack"
import usePlaylist from "../store/PlaylistSlice"
import { PlaylistData } from "./PlaylistData"
import { PlaylistSearchEpisode } from "./PlaylistSearchEpisode"
import { PlaylistSubmitViewer } from "./PlaylistSubmitViewer"
import { components } from "../../schema"
import { $api } from "../utils/http"
import { CustomButtonSecondary } from "./CustomButtonSecondary"
import { CustomButtonPrimary } from "./CustomButtonPrimary"
import { Controller, useForm } from "react-hook-form"

type PlaylistFormValues = {
    id: string
    name: string
    items: components["schemas"]["PodcastEpisodeWithHistory"][]
}

type CreatePlaylistModalProps = {
    open: boolean
    onOpenChange: (open: boolean) => void
    playlistToEdit: components["schemas"]["PlaylistDto"] | undefined
}

export const CreatePlaylistModal: FC<CreatePlaylistModalProps> = ({ open, onOpenChange, playlistToEdit }) => {
    const { t } = useTranslation()
    const playlists = usePlaylist(state => state.playlist)
    const setPlaylist = usePlaylist(state => state.setPlaylist)
    const [stage, setStage] = useState<number>(0)
    const wasOpenRef = useRef(false)
    const createPlaylistMutation = $api.useMutation('post', '/api/v1/playlist')
    const updatePlaylistMutation = $api.useMutation('put', '/api/v1/playlist/{playlist_id}')
    const { handleSubmit, watch, setValue, reset, clearErrors, setError, getValues, control, formState: { errors, isSubmitting } } = useForm<PlaylistFormValues>({
        defaultValues: {
            id: "-1",
            name: "",
            items: []
        }
    })
    const watchedId = watch("id")
    const watchedName = watch("name")
    const watchedItems = watch("items")

    useEffect(() => {
        if (open && !wasOpenRef.current) {
            setStage(0)
            reset({
                id: playlistToEdit?.id ?? "-1",
                name: playlistToEdit?.name ?? "",
                items: playlistToEdit?.items ?? []
            })
        }
        wasOpenRef.current = open
    }, [open, playlistToEdit, reset])

    const nameIsValid = (watchedName?.trim().length ?? 0) > 0
    const hasItems = (watchedItems?.length ?? 0) > 0
    const canMoveToStageTwo = nameIsValid
    const canSubmit = nameIsValid && hasItems && !isSubmitting

    const closeModal = () => {
        onOpenChange(false)
        setStage(0)
    }

    const validateName = () => {
        if (!getValues("name").trim()) {
            setError("name", { type: "required", message: t("playlist-name") })
            return false
        }
        clearErrors("name")
        return true
    }

    const validateItems = () => {
        if (getValues("items").length === 0) {
            setError("items", { type: "required", message: t("available-episodes") })
            return false
        }
        clearErrors("items")
        return true
    }

    const handlePlaylistCreateOrUpdate = handleSubmit(async (values) => {
        if (!validateName() || !validateItems()) {
            return
        }
        const trimmedName = values.name.trim()
        const itemsMappedToIDs = values.items.map(item => ({
            episode: item.podcastEpisode.id
        } satisfies components["schemas"]["PlaylistItem"]))

        try {
            if (values.id !== "-1") {
                const response = await updatePlaylistMutation.mutateAsync({
                    params: {
                        path: {
                            playlist_id: values.id
                        }
                    },
                    body: {
                        name: trimmedName,
                        items: itemsMappedToIDs
                    }
                })
                if (!response?.id) {
                    throw new Error("Playlist update returned empty payload")
                }
                const mapped = playlists.map(p => p.id === response.id ? response : p)
                setPlaylist(mapped)
                enqueueSnackbar(t('updated-playlist'), { variant: "success" })
            } else {
                const response = await createPlaylistMutation.mutateAsync({
                    body: {
                        name: trimmedName,
                        items: itemsMappedToIDs
                    }
                })
                if (!response?.id) {
                    throw new Error("Playlist create returned empty payload")
                }
                setPlaylist([...playlists, response])
                enqueueSnackbar(t('created-playlist'), { variant: "success" })
            }

            closeModal()
        } catch (e) {
            enqueueSnackbar(t('error-occured'), { variant: "error" })
        }
    })

    const goNext = () => {
        if (stage === 0 && !validateName()) {
            return
        }
        if (stage === 1 && !validateItems()) {
            return
        }
        if (stage < 2) {
            setStage(stage + 1)
        }
    }

    const goBack = () => {
        if (stage > 0) {
            setStage(stage - 1)
        }
    }

    return (
        <Dialog
            open={open}
            onOpenChange={(next) => {
                onOpenChange(next)
                if (!next) setStage(0)
            }}
        >
            <DialogContent
                className="max-w-5xl w-full md:w-[70%] max-h-[92vh] overflow-hidden sm:max-w-5xl"
            >
                <form
                    onSubmit={(e) => {
                        e.preventDefault()
                        if (stage < 2) {
                            return
                        }
                        void handlePlaylistCreateOrUpdate()
                    }}
                    onKeyDown={(e) => {
                        if (e.key === "Enter" && stage < 2) {
                            e.preventDefault()
                        }
                    }}
                    className="flex flex-col"
                >
                    <DialogHeader>
                        <DialogTitle>{t('add-playlist')}</DialogTitle>
                        <DialogDescription className="sr-only">
                            {t('add-playlist')}
                        </DialogDescription>
                        <div className="text-xs text-muted-foreground">{stage + 1} / 3</div>
                    </DialogHeader>

                    <div className="overflow-y-auto pr-1 max-h-[62vh] md:max-h-[58vh] mt-4">
                        {stage === 0 && (
                            <Controller
                                name="name"
                                control={control}
                                render={({ field }) => (
                                    <PlaylistData
                                        name={field.value ?? ""}
                                        onNameChange={(name) => {
                                            field.onChange(name)
                                            if (name.trim().length > 0) {
                                                clearErrors("name")
                                            }
                                        }}
                                    />
                                )}
                            />
                        )}
                        {stage === 1 && (
                            <Controller
                                name="items"
                                control={control}
                                render={({ field }) => (
                                    <PlaylistSearchEpisode
                                        items={field.value ?? []}
                                        setItems={(items) => {
                                            field.onChange(items)
                                            if (items.length > 0) {
                                                clearErrors("items")
                                            }
                                        }}
                                    />
                                )}
                            />
                        )}
                        {stage === 2 && <PlaylistSubmitViewer playlistName={watchedName?.trim() ?? ""} episodeCount={watchedItems?.length ?? 0} />}
                        {(errors.name?.message || errors.items?.message) && (
                            <div className="mt-3 text-xs text-destructive">{errors.name?.message || errors.items?.message}</div>
                        )}
                    </div>

                    <div className="mt-4 pt-3 flex items-center justify-between border-t border-border">
                        {stage === 0 ? (
                            <CustomButtonSecondary type="button" onClick={closeModal}>
                                {t('cancel')}
                            </CustomButtonSecondary>
                        ) : (
                            <CustomButtonSecondary type="button" onClick={goBack}>
                                <span className="inline-flex items-center gap-1">
                                    <span className="material-symbols-outlined !text-base leading-none">arrow_back</span>
                                    <span>{t('back')}</span>
                                </span>
                            </CustomButtonSecondary>
                        )}
                        {stage < 2 ? (
                            <CustomButtonPrimary
                                type="button"
                                onClick={goNext}
                                disabled={stage === 0 && !canMoveToStageTwo}
                            >
                                <span className="inline-flex items-center gap-1">
                                    <span>{t('next')}</span>
                                    <span className="material-symbols-outlined !text-base leading-none">arrow_forward</span>
                                </span>
                            </CustomButtonPrimary>
                        ) : (
                            <CustomButtonPrimary type="submit" loading={isSubmitting} disabled={!canSubmit}>
                                {watchedId === "-1" ? t('create-playlist') : t('update-playlist')}
                            </CustomButtonPrimary>
                        )}
                    </div>
                </form>
            </DialogContent>
        </Dialog>
    )
}
