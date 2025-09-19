import {createRef, FC, useState} from 'react'
import {Link} from 'react-router-dom'
import {prependAPIKeyOnAuthEnabled} from '../utils/Utilities'
import useCommon, {Podcast} from '../store/CommonSlice'
import 'material-symbols/outlined.css'
import * as Context from '@radix-ui/react-context-menu'
import {CustomInput} from "./CustomInput";
import {PlusIcon} from "../icons/PlusIcon";
import {CustomCheckbox} from "./CustomCheckbox";
import {LuTags} from "react-icons/lu";
import {useTranslation} from "react-i18next";
import {components} from "../../schema";
import {$api, client} from "../utils/http";
import {Loading} from "./Loading";

type PodcastCardProps = {
    podcast: components["schemas"]["PodcastDto"]
}

export const PodcastCard: FC<PodcastCardProps> = ({podcast}) => {
    const likeButton = createRef<HTMLElement>()
    const updateLikePodcast = useCommon(state => state.updateLikePodcast)
    const tags = useCommon(state=>state.tags)
    const setTags = useCommon(state=>state.setPodcastTags)
    const setPodcasts = useCommon(state=>state.setPodcasts)
    const podcasts = useCommon(state=>state.podcasts)
    const likePodcast = () => {
        client.PUT("/api/v1/podcasts/favored", {
            body: {
                id: podcast.id,
                favored: !podcast.favorites
            }
        })
    }
    const {t} = useTranslation()
    const [newTag, setNewTag] = useState<string>('')
    const {data, error, isLoading} = $api.useQuery('get', '/api/v1/users/{username}', {
        params: {
            path: {
                username: 'me'
            }
        },
    })


    if (isLoading ||!data) {
        return <Loading/>
    }


    return (
        <Context.Root modal={true} onOpenChange={()=>{
            setNewTag('')
        }}>
            <Context.Trigger>
                <Link className="group" to={podcast.id + '/episodes'}>
                    <div className="relative mb-2">
                        <img
                            className={`rounded-xl transition-shadow group-hover:shadow-[0_4px_32px_rgba(0,0,0,var(--shadow-opacity))] ${!podcast.active ? 'opacity-20' : ''}`}
                            src={prependAPIKeyOnAuthEnabled(podcast.image_url, data)} alt=""/>

                        <span ref={likeButton}
                              className={`material-symbols-outlined filled absolute top-2 right-2 h-6 w-6 filled ${podcast.favorites ? 'text-rose-700 hover:text-rose-600' : 'text-stone-200 hover:text-stone-100'}`}
                              onClick={(e) => {
                                  // Prevent icon click from triggering link to podcast detail
                                  e.preventDefault()

                                  likeButton.current?.classList.toggle('fill-amber-400')
                                  likePodcast()
                                  updateLikePodcast(podcast.id)
                              }}>favorite</span>
                    </div>

                    <div>
                        <span
                            className="block font-bold leading-[1.2] mb-2 text-(--fg-color) transition-colors group-hover:text-(--fg-color-hover)">{podcast.name}</span>
                        <span
                            className="block leading-[1.2] text-sm text-(--fg-secondary-color)">{podcast.author}</span>
                        <span className="flex gap-2 mb-2 text-(--fg-color)"><LuTags className="text-(--fg-secondary-color) text-2xl"/> <span className="self-center mb-2 text-(--fg-color)">{podcast.tags.length}</span> {t('tag', {count: tags.length})}</span>
                    </div>
                </Link>
            </Context.Trigger>
            <Context.Portal>
                <Context.Content className="bg-(--bg-color) p-5" onClick={(e)=>{
                    e.preventDefault()
                }}>
                    <h2 className="text-(--fg-color)">Tags</h2>
                    <hr className="mt-1 border-[1px] border-(--border-color) mb-2"/>
                    <div className="flex gap-3 flex-col text-(--fg-color)">
                    {
                     tags.map(t=>{
                         return <Context.Item key={t.id} onClick={(e)=>{
                             e.preventDefault()
                         }} className="">
                             <span className="grid grid-cols-[auto_1fr] gap-5">
                                 <CustomCheckbox value={podcast.tags.filter(e=>e.name=== t.name).length>0} onChange={(v)=>{
                                     if (v.valueOf() === true) {
                                         client.POST("/api/v1/tags/{tag_id}/{podcast_id}", {
                                             params: {
                                                 path: {
                                                        tag_id: t.id,
                                                        podcast_id: podcast.id
                                                 }
                                             }
                                         }).then(()=>{
                                             const addedTag = tags.filter(tag=>tag.id === t.id)[0]!
                                             setPodcasts(podcasts.map(p=>{
                                                 if (p.id === podcast.id) {
                                                     const tags = podcast.tags
                                                     tags.push(addedTag)
                                                     return {
                                                         ...p,
                                                         tags
                                                     }
                                                 }
                                                 return p
                                             }))
                                         })
                                     } else {
                                         client.DELETE("/api/v1/tags/{tag_id}/{podcast_id}", {
                                             params: {
                                                 path: {
                                                     tag_id: t.id,
                                                     podcast_id: podcast.id
                                                 }
                                             }
                                         }).then(()=>{
                                             setPodcasts(podcasts.map(p=>{
                                                 if (p.id === podcast.id) {
                                                     const tags = podcast.tags.filter(tag=>tag.id !== t.id)
                                                     return {
                                                         ...p,
                                                         tags
                                                     }
                                                 }
                                                 return p
                                             }))
                                         })
                                     }
                                 }}/>
                                 <span className="">{t.name}</span>
                             </span>
                         </Context.Item>
                     })
                    }
                    </div>

                    <span className="relative">
                        <PlusIcon className="absolute right-5 fill-white h-[19px] top-2  -translate-y-1/2 cursor-pointer" onClick={()=>{
                            if(tags.map(t=>t.name).includes(newTag)||!newTag.trim()) {
                                return
                            }

                            client.POST('/api/v1/tags', {
                                body: {
                                    name: newTag,
                                    color: 'Green'
                                }
                            }).then((resp)=>{
                                if (resp) {
                                    setTags([...tags,resp.data!])
                                }
                            })
                        }}/>
                        <CustomInput className="mt-5" placeholder="Add new tag" value={newTag} onChange={(event)=>{
                            setNewTag(event.target.value)
                        }}/>
                    </span>
                </Context.Content>
            </Context.Portal>
        </Context.Root>
    )
}
