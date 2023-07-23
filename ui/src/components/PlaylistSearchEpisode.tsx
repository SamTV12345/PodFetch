import {EpisodeSearch} from "./EpisodeSearch";
import {DragEvent, useState} from "react";
import {PodcastEpisode} from "../store/CommonSlice";
import {useTranslation} from "react-i18next";
import {useAppDispatch, useAppSelector} from "../store/hooks";
import {setCurrentPlaylistToEdit} from "../store/PlaylistSlice";



export const PlaylistSearchEpisode = ()=>{
    const [itemCurrentlyDragged,setItemCurrentlyDragged] = useState<PodcastEpisode>()
    const {t} = useTranslation()
    const dispatch = useAppDispatch()
    const currentPlayListToEdit = useAppSelector(state => state.playlist.currentPlaylistToEdit)
    const handleDragStart = (dragItem: PodcastEpisode, index: number, event: DragEvent<HTMLTableRowElement> )=>{
        event.dataTransfer.setData("text/plain", index.toString())
        setItemCurrentlyDragged(dragItem)
    }

    return        <>
        <EpisodeSearch onClickResult={e=>{
            dispatch(setCurrentPlaylistToEdit({
                id: currentPlayListToEdit!.id,
                name: currentPlayListToEdit!.name,
                items: [...currentPlayListToEdit!.items, e]
            }))
        }} classNameResults="max-h-[min(20rem,calc(100vh-3rem-3rem))]"
                                                  showBlankState={false} />
    <div className={`scrollbox-x`}>
        <table className="text-left text-sm text-stone-900 w-full overflow-y-auto">
            <thead>
            <tr className="border-b border-stone-300">
                <th scope="col" className="pr-2 py-3">
                    #
                </th>
                <th scope="col" className="px-2 py-3">
                    {t('episode-name')}
                </th>
                <th scope="col" className="px-2 py-3">
                    {t('actions')}
                </th>
            </tr>
            </thead>
            <tbody className="">
            {currentPlayListToEdit?.items.map((item, index) => {
                return <tr draggable onDrop={e=>{
                    e.preventDefault()
                    const dropIndex = index
                    const dragIndex = parseInt(e.dataTransfer.getData("text/plain"))

                    const newItems = [...currentPlayListToEdit!.items]
                    const dragItem = newItems[dragIndex]
                    newItems.splice(dragIndex, 1)
                    newItems.splice(dropIndex, 0, dragItem)
                    dispatch(setCurrentPlaylistToEdit({
                        name: currentPlayListToEdit!.name,
                        id: currentPlayListToEdit!.id,
                        items: newItems
                    }))
                }} onDragOver={(e)=>item.id!=itemCurrentlyDragged?.id&&e.preventDefault()} onDragStart={e=>handleDragStart(item, index, e)}>
                    <td>
                        {index}
                    </td>
                    <td>
                        {item.name}
                    </td>
                    <td>
                        <button className="flex text-red-700 hover:text-red-500" onClick={e=>{
                            e.preventDefault()
                            dispatch(setCurrentPlaylistToEdit({
                                name: currentPlayListToEdit!.name,
                                id: currentPlayListToEdit!.id,
                                items: currentPlayListToEdit!.items.filter(i=>i.id!==item.id)
                            }))
                        }}><span className="material-symbols-outlined mr-1">delete</span>
                            {t('delete')}</button>
                    </td>
                </tr>
            })}
            </tbody>
        </table>
    </div>
    </>
}
