import {FC, useMemo, useRef, useState} from 'react'
import {Link} from 'react-router-dom'
import 'material-symbols/outlined.css'
import * as Popover from '@radix-ui/react-popover'
import {CustomInput} from "./CustomInput";
import {CustomCheckbox} from "./CustomCheckbox";
import {useTranslation} from "react-i18next";
import {components} from "../../schema";
import {$api} from "../utils/http";
import {useQueryClient} from "@tanstack/react-query";

type PodcastCardProps = {
    podcast: components["schemas"]["PodcastDto"]
}

const MAX_VISIBLE_TAGS = 3

export const PodcastCard: FC<PodcastCardProps> = ({podcast}) => {
    const likeButton = useRef<HTMLSpanElement>(null)
    const tags = $api.useQuery('get', '/api/v1/tags')
    const queryClient = useQueryClient()
    const toggleFavoriteMutation = $api.useMutation('put', '/api/v1/podcasts/favored')
    const addTagToPodcastMutation = $api.useMutation('post', '/api/v1/tags/{tag_id}/{podcast_id}')
    const removeTagFromPodcastMutation = $api.useMutation('delete', '/api/v1/tags/{tag_id}/{podcast_id}')
    const createTagMutation = $api.useMutation('post', '/api/v1/tags')
    const {t} = useTranslation()
    const [newTag, setNewTag] = useState<string>('')
    const [tagEditorOpen, setTagEditorOpen] = useState<boolean>(false)
    const availableTags = tags.data ?? []
    const selectedTagIds = useMemo(() => new Set(podcast.tags.map(tag => tag.id)), [podcast.tags])
    const tagNameAlreadyExists = availableTags.some((tag) => tag.name.toLowerCase() === newTag.trim().toLowerCase())

    const updatePodcastSearchCaches = (updater: (p: components["schemas"]["PodcastDto"]) => components["schemas"]["PodcastDto"]) => {
        queryClient.setQueriesData<components["schemas"]["PodcastDto"][]>({queryKey: ['get', '/api/v1/podcasts/search']}, (oldData) => {
            if (!oldData) {
                return oldData
            }
            return oldData.map(p => p.id === podcast.id ? updater(p) : p)
        })
    }

    const likePodcast = () => {
        toggleFavoriteMutation.mutate({
            body: {
                id: podcast.id,
                favored: !podcast.favorites
            }
        })
        updatePodcastSearchCaches((p) => ({
            ...p,
            favorites: !p.favorites
        }))
    }

    const togglePodcastTag = (tag: (typeof availableTags)[number], checked: boolean) => {
        if (checked && !selectedTagIds.has(tag.id)) {
            addTagToPodcastMutation.mutateAsync({
                params: {
                    path: {
                        tag_id: tag.id,
                        podcast_id: podcast.id
                    }
                }
            }).then(() => {
                updatePodcastSearchCaches((p) => {
                    if (p.tags.some(existingTag => existingTag.id === tag.id)) {
                        return p
                    }

                    return {
                        ...p,
                        tags: [...p.tags, tag]
                    }
                })
            })
            return
        }

        if (!checked && selectedTagIds.has(tag.id)) {
            removeTagFromPodcastMutation.mutateAsync({
                params: {
                    path: {
                        tag_id: tag.id,
                        podcast_id: podcast.id
                    }
                }
            }).then(() => {
                updatePodcastSearchCaches((p) => ({
                    ...p,
                    tags: p.tags.filter(existingTag => existingTag.id !== tag.id)
                }))
            })
        }
    }

    const createAndAssignTag = () => {
        const normalizedTag = newTag.trim()
        if (!normalizedTag || tagNameAlreadyExists) {
            return
        }

        createTagMutation.mutateAsync({
            body: {
                name: normalizedTag,
                color: 'Green'
            }
        }).then((createdTag) => {
            setNewTag('')
            queryClient.invalidateQueries({queryKey: ['get', '/api/v1/tags']})

            if (!createdTag) {
                return
            }

            addTagToPodcastMutation.mutateAsync({
                params: {
                    path: {
                        tag_id: createdTag.id,
                        podcast_id: podcast.id
                    }
                }
            }).then(() => {
                updatePodcastSearchCaches((p) => {
                    if (p.tags.some(existingTag => existingTag.id === createdTag.id)) {
                        return p
                    }

                    return {
                        ...p,
                        tags: [...p.tags, createdTag]
                    }
                })
            })
        })
    }

    return (
        <div className="group">
            <Link className="block" to={podcast.id + '/episodes'}>
                <div className="relative mb-2">
                    <img
                        className={`rounded-xl transition-shadow group-hover:shadow-[0_4px_32px_rgba(0,0,0,var(--shadow-opacity))] ${!podcast.active ? 'opacity-20' : ''}`}
                        src={podcast.image_url} alt=""/>

                    <span ref={likeButton}
                          className={`material-symbols-outlined filled absolute top-2 right-2 h-6 w-6 filled ${podcast.favorites ? 'text-rose-700 hover:text-rose-600' : 'text-stone-200 hover:text-stone-100'}`}
                          onClick={(e) => {
                              // Prevent icon click from triggering link to podcast detail
                              e.preventDefault()

                              likeButton.current?.classList.toggle('fill-amber-400')
                              likePodcast()
                          }}>favorite</span>
                </div>

                <div>
                    <span
                        className="block font-bold leading-[1.2] mb-2 ui-text transition-colors group-hover:ui-text-hover">{podcast.name}</span>
                    <span
                        className="block leading-[1.2] text-sm ui-text-muted">{podcast.author}</span>
                </div>
            </Link>

            <div className="mt-2 flex items-center justify-between gap-2">
                <div className="flex flex-wrap gap-1">
                    {podcast.tags.slice(0, MAX_VISIBLE_TAGS).map((tag) => (
                        <span key={tag.id} className="inline-flex items-center rounded-md ui-input-surface px-2 py-0.5 text-xs ui-text">
                            {tag.name}
                        </span>
                    ))}
                    {podcast.tags.length > MAX_VISIBLE_TAGS && (
                        <span className="inline-flex items-center rounded-md ui-input-surface px-2 py-0.5 text-xs ui-text-muted">
                            +{podcast.tags.length - MAX_VISIBLE_TAGS}
                        </span>
                    )}
                </div>

                <Popover.Root modal={true} open={tagEditorOpen} onOpenChange={(open) => {
                    setTagEditorOpen(open)
                    if (!open) {
                        setNewTag('')
                    }
                }}>
                    <Popover.Trigger asChild>
                        <button className="inline-flex shrink-0 items-center gap-1 rounded-md border ui-border px-2 py-1 text-xs ui-text hover:ui-input-surface hover:ui-text-hover" type="button">
                            <span className="material-symbols-outlined text-sm leading-none">sell</span>
                            {t('tag_other')}
                        </button>
                    </Popover.Trigger>

                    <Popover.Portal>
                        <Popover.Content align="end" side="top" sideOffset={8}
                                         className="w-72 ui-surface p-4 rounded-lg shadow-[0_4px_16px_rgba(0,0,0,var(--shadow-opacity))] z-30">
                            <div className="mb-3 flex items-center justify-between">
                                <h3 className="font-semibold ui-text">{t('tag_other')}</h3>
                                <span className="text-xs ui-text-muted">{podcast.tags.length}</span>
                            </div>

                            <div className="max-h-48 overflow-y-auto pr-1">
                                {availableTags.length === 0 ? (
                                    <p className="text-sm ui-text-muted">{t('no')}</p>
                                ) : (
                                    <div className="flex flex-col gap-1">
                                        {availableTags.map((tag) => (
                                            <label key={tag.id}
                                                   className="grid grid-cols-[auto_1fr] items-center gap-3 rounded-md px-2 py-1 hover:ui-input-surface cursor-pointer">
                                                <CustomCheckbox
                                                    value={selectedTagIds.has(tag.id)}
                                                    onChange={(checked) => togglePodcastTag(tag, checked === true)}
                                                />
                                                <span className="text-sm ui-text">{tag.name}</span>
                                            </label>
                                        ))}
                                    </div>
                                )}
                            </div>

                            <div className="mt-3 grid grid-cols-[1fr_auto] items-center gap-2">
                                <CustomInput
                                    className="py-1.5"
                                    placeholder={t('tag-add-placeholder') as string}
                                    value={newTag}
                                    onChange={(event) => {
                                        setNewTag(event.target.value)
                                    }}
                                    onKeyDown={(event) => {
                                        if (event.key === 'Enter') {
                                            createAndAssignTag()
                                        }
                                    }}
                                />
                                <button
                                    className="inline-flex h-8 w-8 items-center justify-center rounded-md ui-bg-accent hover:ui-bg-accent-hover disabled:opacity-50"
                                    type="button"
                                    onClick={createAndAssignTag}
                                    disabled={createTagMutation.isPending || !newTag.trim() || tagNameAlreadyExists}
                                >
                                    <span className="material-symbols-outlined text-sm ui-text-inverse">add</span>
                                </button>
                            </div>

                            <Popover.Arrow className="ui-fill-inverse"/>
                        </Popover.Content>
                    </Popover.Portal>
                </Popover.Root>
            </div>
        </div>
    )
}
