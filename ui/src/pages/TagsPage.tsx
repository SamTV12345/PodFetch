import {Heading1} from "../components/Heading1";
import {useTranslation} from "react-i18next";
import {useMemo, useState} from "react";
import {CustomInput} from "../components/CustomInput";
import {$api} from "../utils/http";
import {LoadingSkeletonSpan} from "../components/ui/LoadingSkeletonSpan";
import {useQueryClient} from "@tanstack/react-query";
import {CustomButtonPrimary} from "../components/CustomButtonPrimary";
import {CustomButtonSecondary} from "../components/CustomButtonSecondary";
import {CustomSelect, Option} from "../components/CustomSelect";
import {components} from "../../schema";

type TagColor = "Green" | "Red" | "Blue"

type TagDraft = {
    name: string,
    color: TagColor
}

const tagColorValues: { value: TagColor, labelKey: string }[] = [
    {value: 'Green', labelKey: 'color-green'},
    {value: 'Red', labelKey: 'color-red'},
    {value: 'Blue', labelKey: 'color-blue'}
]

const normalizeTagColor = (value: string): TagColor => {
    if (value === 'Red' || value === 'Blue' || value === 'Green') {
        return value
    }
    return 'Green'
}

const tagDotColorClass = (color: string) => {
    switch (normalizeTagColor(color)) {
        case 'Red':
            return 'bg-red-500'
        case 'Blue':
            return 'bg-blue-500'
        default:
            return 'bg-green-500'
    }
}

export const TagsPage = ()=>{
    const {t}  = useTranslation()
    const tags = $api.useQuery('get', '/api/v1/tags')
    const queryClient = useQueryClient()
    const createTagMutation = $api.useMutation('post', '/api/v1/tags')
    const updateTagMutation = $api.useMutation('put', '/api/v1/tags/{tag_id}')
    const deleteTagMutation = $api.useMutation('delete', '/api/v1/tags/{tag_id}')
    const [newTagName, setNewTagName] = useState<string>('')
    const [newTagColor, setNewTagColor] = useState<TagColor>('Green')
    const [search, setSearch] = useState<string>('')
    const [drafts, setDrafts] = useState<Record<string, TagDraft>>({})

    const availableTags = tags.data ?? []
    const tagColorOptions: Option[] = useMemo(() => tagColorValues.map((color) => ({
        value: color.value,
        label: t(color.labelKey)
    })), [t])

    const filteredTags = useMemo(() => {
        const query = search.trim().toLowerCase()
        if (!query) {
            return availableTags
        }

        return availableTags.filter(tag => tag.name.toLowerCase().includes(query))
    }, [availableTags, search])

    const setTagsCache = (updater: (prev: components["schemas"]["Tag"][]) => components["schemas"]["Tag"][]) => {
        queryClient.setQueryData(['get', '/api/v1/tags'], (oldData?: components["schemas"]["Tag"][]) => {
            if (!oldData) {
                return oldData
            }
            return updater(oldData)
        })
    }

    const getTagKey = (id: string | number) => String(id)

    const getTagDraft = (tag: components["schemas"]["Tag"]): TagDraft => {
        return drafts[getTagKey(tag.id)] ?? {
            name: tag.name,
            color: normalizeTagColor(tag.color)
        }
    }

    const updateTagDraft = (id: string | number, patch: Partial<TagDraft>) => {
        const key = getTagKey(id)
        setDrafts((prev) => ({
            ...prev,
            [key]: {
                ...(prev[key] ?? {name: '', color: 'Green'}),
                ...patch
            } as TagDraft
        }))
    }

    const clearTagDraft = (id: string | number) => {
        const key = getTagKey(id)
        setDrafts((prev) => {
            if (!(key in prev)) {
                return prev
            }
            const next = {...prev}
            delete next[key]
            return next
        })
    }

    const isDuplicateName = (name: string, currentTagId?: string | number) => {
        const normalized = name.trim().toLowerCase()
        if (!normalized) {
            return false
        }

        return availableTags.some((tag) => {
            if (currentTagId !== undefined && String(tag.id) === String(currentTagId)) {
                return false
            }
            return tag.name.trim().toLowerCase() === normalized
        })
    }

    const createTagDisabled = !newTagName.trim() || isDuplicateName(newTagName) || createTagMutation.isPending

    const createTag = () => {
        const normalizedName = newTagName.trim()
        if (!normalizedName || isDuplicateName(normalizedName)) {
            return
        }

        createTagMutation.mutateAsync({
            body: {
                name: normalizedName,
                color: newTagColor
            }
        }).then((createdTag) => {
            if (!createdTag) {
                return
            }

            setTagsCache((oldTags) => [...oldTags, createdTag])
            setNewTagName('')
            setNewTagColor('Green')
        })
    }

    const saveTag = (tag: components["schemas"]["Tag"]) => {
        const draft = getTagDraft(tag)
        const normalizedName = draft.name.trim()
        const color = normalizeTagColor(draft.color)
        if (!normalizedName || isDuplicateName(normalizedName, tag.id)) {
            return
        }

        updateTagMutation.mutate({
            params: {
                path: {
                    tag_id: tag.id
                }
            },
            body: {
                name: normalizedName,
                color
            }
        }, {
            onSuccess: () => {
                setTagsCache((oldTags) => oldTags.map((existingTag) => {
                    if (String(existingTag.id) !== String(tag.id)) {
                        return existingTag
                    }
                    return {
                        ...existingTag,
                        name: normalizedName,
                        color
                    }
                }))
                clearTagDraft(tag.id)
            }
        })
    }

    const deleteTag = (tag: components["schemas"]["Tag"]) => {
        deleteTagMutation.mutate({
            params: {
                path: {
                    tag_id: tag.id
                }
            }
        }, {
            onSuccess: () => {
                setTagsCache((oldTags) => oldTags.filter((existingTag) => String(existingTag.id) !== String(tag.id)))
                clearTagDraft(tag.id)
                queryClient.invalidateQueries({queryKey: ['get', '/api/v1/podcasts/search']})
            }
        })
    }

    return (
        <div>
            <Heading1>{t('tag_other')}</Heading1>

            <div className="mt-6 mb-6 grid gap-3 md:grid-cols-[1fr_auto_auto]">
                <CustomInput
                    className="w-full"
                    value={search}
                    onChange={(event) => setSearch(event.target.value)}
                    placeholder={t('search') as string}
                />
                <CustomInput
                    className="w-full md:min-w-56"
                    value={newTagName}
                    onChange={(event) => setNewTagName(event.target.value)}
                    placeholder={t('tag-add-placeholder') as string}
                    onKeyDown={(event) => {
                        if (event.key === 'Enter') {
                            createTag()
                        }
                    }}
                />
                <div className="grid grid-cols-[8rem_auto] gap-2">
                    <CustomSelect
                        options={tagColorOptions}
                        value={newTagColor}
                        onChange={(value) => setNewTagColor(normalizeTagColor(value))}
                    />
                    <CustomButtonPrimary disabled={createTagDisabled} onClick={createTag}>
                        {t('add')}
                    </CustomButtonPrimary>
                </div>
            </div>

            <div className="rounded-xl border ui-border overflow-hidden">
                {(tags.isLoading || !tags.data) ? (
                    Array.from({length: 5}).map((_, index) => (
                        <div key={index} className="grid gap-2 p-3 border-b ui-border-b md:grid-cols-[1fr_auto_auto_auto]">
                            <LoadingSkeletonSpan height="30px" loading={true}/>
                            <LoadingSkeletonSpan height="30px" loading={true}/>
                            <LoadingSkeletonSpan height="30px" loading={true}/>
                            <LoadingSkeletonSpan height="30px" loading={true}/>
                        </div>
                    ))
                ) : filteredTags.length === 0 ? (
                    <div className="p-4 text-sm ui-text-muted">{t('no-results-found-for')} "{search}"</div>
                ) : (
                    filteredTags.map((tag) => {
                        const draft = getTagDraft(tag)
                        const normalizedName = draft.name.trim()
                        const hasChanges = normalizedName !== tag.name || normalizeTagColor(draft.color) !== normalizeTagColor(tag.color)
                        const saveDisabled = !hasChanges || !normalizedName || isDuplicateName(normalizedName, tag.id) || updateTagMutation.isPending

                        return <div className="grid items-center gap-2 p-3 border-b last:border-b-0 ui-border-b md:grid-cols-[1fr_8rem_auto_auto]" key={tag.id}>
                            <label className="grid grid-cols-[auto_1fr] items-center gap-2">
                                <span className={`h-2.5 w-2.5 rounded-full ${tagDotColorClass(draft.color)}`}></span>
                                <CustomInput
                                    value={draft.name}
                                    onChange={(event) => updateTagDraft(tag.id, {name: event.target.value})}
                                    onKeyDown={(event) => {
                                        if (event.key === 'Enter' && !saveDisabled) {
                                            saveTag(tag)
                                        }
                                    }}
                                />
                            </label>
                            <CustomSelect
                                options={tagColorOptions}
                                value={draft.color}
                                onChange={(value) => updateTagDraft(tag.id, {color: normalizeTagColor(value)})}
                            />
                            <CustomButtonSecondary disabled={saveDisabled} onClick={() => saveTag(tag)}>
                                {t('save')}
                            </CustomButtonSecondary>
                            <button
                                className="px-3 py-2 rounded-md ui-text-inverse bg-red-700 hover:bg-red-600 disabled:opacity-50"
                                onClick={() => deleteTag(tag)}
                                disabled={deleteTagMutation.isPending}
                            >
                                {t('delete')}
                            </button>
                        </div>
                    })
                )}
            </div>
        </div>
    )
}
