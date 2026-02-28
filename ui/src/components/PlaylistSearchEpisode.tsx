import {DragEvent, useMemo, useState} from "react";
import {useTranslation} from "react-i18next";
import {components} from "../../schema";
import {EpisodeSearch} from "./EpisodeSearch";

type PlaylistSearchEpisodeProps = {
    items: components["schemas"]["PodcastEpisodeWithHistory"][]
    setItems: (items: components["schemas"]["PodcastEpisodeWithHistory"][]) => void
}

export const PlaylistSearchEpisode = ({items, setItems}: PlaylistSearchEpisodeProps)=> {
    const {t} = useTranslation()
    const [itemCurrentlyDragged, setItemCurrentlyDragged] = useState<components["schemas"]["PodcastEpisodeDto"]>()

    const episodeIds = useMemo(() => {
        return new Set(items.map(item => item.podcastEpisode.id))
    }, [items])

    const moveItem = (from: number, to: number) => {
        if (from < 0 || to < 0 || from >= items.length || to >= items.length) {
            return
        }
        const newItems = [...items]
        const dragged = newItems[from]!
        newItems.splice(from, 1)
        newItems.splice(to, 0, dragged)
        setItems(newItems)
    }

    const addEpisode = (episode: components["schemas"]["PodcastEpisodeDto"]) => {
        if (episodeIds.has(episode.id)) {
            return
        }
        setItems([...items, {podcastEpisode: episode}])
    }

    return (
        <>
            <EpisodeSearch
                onClickResult={addEpisode}
                classNameResults="min-h-[12rem]"
                resultsMaxHeight="16rem"
                showBlankState={false}
            />

            <div className="mt-4 rounded-xl border ui-border">
                <div className="flex items-center justify-between border-b ui-border px-4 py-3">
                    <span className="font-medium">{t('playlists')}</span>
                    <span className="text-xs ui-text-muted">
                        {t('item_other', {count: items.length})}
                    </span>
                </div>
                <div className="max-h-[18rem] overflow-y-auto">
                    {!items.length && (
                        <div className="p-4 text-sm ui-text-muted">
                            {t('playlist-search-help')}
                        </div>
                    )}
                    {items.map((item, index) => (
                        <div
                            key={`${item.podcastEpisode.id}-${index}`}
                            className="grid grid-cols-[2rem_1fr_auto] items-center gap-2 border-b ui-border px-3 py-2"
                            draggable
                            onDragStart={(event: DragEvent<HTMLDivElement>) => {
                                event.dataTransfer.setData("text/plain", index.toString())
                                setItemCurrentlyDragged(item.podcastEpisode)
                            }}
                            onDragOver={(event) => {
                                if (item.podcastEpisode.id !== itemCurrentlyDragged?.id) {
                                    event.preventDefault()
                                }
                            }}
                            onDrop={(event) => {
                                event.preventDefault()
                                const dragIndex = parseInt(event.dataTransfer.getData("text/plain"))
                                moveItem(dragIndex, index)
                            }}
                        >
                            <span className="text-xs ui-text-muted">{index + 1}</span>
                            <div className="min-w-0">
                                <div className="line-clamp-1 text-sm font-medium">{item.podcastEpisode.name}</div>
                            </div>
                            <div className="flex items-center gap-1 self-center">
                                <button
                                    type="button"
                                    className="h-7 w-7 grid place-items-center material-symbols-outlined leading-none ui-icon-muted hover:ui-text"
                                    onClick={(e) => {
                                        e.preventDefault()
                                        moveItem(index, index - 1)
                                    }}
                                    disabled={index === 0}
                                >
                                    arrow_upward
                                </button>
                                <button
                                    type="button"
                                    className="h-7 w-7 grid place-items-center material-symbols-outlined leading-none ui-icon-muted hover:ui-text"
                                    onClick={(e) => {
                                        e.preventDefault()
                                        moveItem(index, index + 1)
                                    }}
                                    disabled={index === items.length - 1}
                                >
                                    arrow_downward
                                </button>
                                <button
                                    type="button"
                                    className="h-7 w-7 grid place-items-center material-symbols-outlined leading-none ui-text-danger hover:ui-text-danger-hover"
                                    onClick={(e) => {
                                        e.preventDefault()
                                        setItems(items.filter((_, i) => i !== index))
                                    }}
                                >
                                    delete
                                </button>
                            </div>
                        </div>
                    ))}
                </div>
            </div>
        </>
    )
}
